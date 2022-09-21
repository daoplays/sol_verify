use std::str::FromStr;
use crate::state::{ProgramMetaData};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::native_token::LAMPORTS_PER_SOL;
use crate::accounts;
use crate::utils;
use crate::state;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    msg,
    pubkey::Pubkey,
    bpf_loader_upgradeable::{self, UpgradeableLoaderState}
};

use crate::{instruction::{VerifyInstruction, SubmitProgramMeta, VerifyProgramMeta}};

pub struct Processor;
impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
        msg!("process!");
        let instruction = VerifyInstruction::try_from_slice(&instruction_data[..])?;

        match instruction {
            VerifyInstruction::SubmitProgram {metadata} => {
                msg!("Instruction: Submit Program");
                Self::submit_program(accounts, program_id, metadata)
            },
            VerifyInstruction::VerifyProgram {metadata} => {
                msg!("Instruction: Verify Program");
                Self::verify_program(accounts, program_id, metadata)
            }
        }
    } 
 
    fn submit_program(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        metadata : SubmitProgramMeta
    ) ->ProgramResult 
    {

        let account_info_iter = &mut accounts.iter();

        // This function expects to be passed eight accounts, get them all first and then check their value is as expected
        let program_owner_account_info = next_account_info(account_info_iter)?;
        let program_metadata_account_info = next_account_info(account_info_iter)?;

        let system_program_account_info = next_account_info(account_info_iter)?;

        // the first account should be the funding account and should be a signer
        if !program_owner_account_info.is_signer {
            msg!("expected first account as signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        let (expected_metadata_key, bump_seed) = Pubkey::find_program_address(&[&metadata.address.to_bytes()], &program_id);
        
        if program_metadata_account_info.key != &expected_metadata_key
        { 
            msg!("expected second account to be the program metadata account {}", expected_metadata_key);
            return Err(ProgramError::InvalidAccountData); 
        }
        
        // the third and final account is the system_program
        if system_program_account_info.key != &solana_program::system_program::id() {
            msg!("expected third account to be the system program {}", solana_program::system_program::id());
            return Err(ProgramError::InvalidAccountData);
        }
        
        // create the users data account if we need it
        utils::create_program_data_account(
            program_owner_account_info,
            program_metadata_account_info,
            program_id,
            bump_seed,
            &metadata.address.to_bytes(),
            state::get_metadata_size()
        )?;

        Ok(())

    }

    fn verify_program(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        metadata : VerifyProgramMeta
    ) ->ProgramResult {

        let account_info_iter = &mut accounts.iter();

        // This function expects to be passed three accounts, get them all first and then check their value is as expected
        let dao_plays_account_info = next_account_info(account_info_iter)?;
        let program_metadata_account_info = next_account_info(account_info_iter)?;

        let real_program_account_info = next_account_info(account_info_iter)?;
        let test_program_account_info = next_account_info(account_info_iter)?;

        let real_program_data_account_info = next_account_info(account_info_iter)?;
        let test_program_data_account_info = next_account_info(account_info_iter)?;



        // the first account should be the funding account and should be a signer
        if !dao_plays_account_info.is_signer {
            msg!("expected first account as signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        // the third account is the daoplays SOL address
        if dao_plays_account_info.key != &accounts::get_expected_daoplays_key()
        {
            msg!("expected third account to be the daoplays address {}", accounts::get_expected_daoplays_key());
            return Err(ProgramError::InvalidAccountData);
        }

        if &metadata.real_address != real_program_account_info.key {
            return Err(ProgramError::InvalidAccountData); 
        }

        if &metadata.test_address != test_program_account_info.key {
            return Err(ProgramError::InvalidAccountData); 
        }

        let (expected_metadata_key, _bump_seed) = Pubkey::find_program_address(&[&metadata.real_address.to_bytes()], &program_id);
        
        if program_metadata_account_info.key != &expected_metadata_key
        { 
            msg!("expected second account to be the program metadata account {}", expected_metadata_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        let data_offset = UpgradeableLoaderState::size_of_programdata_metadata();

        let real_program_data = &real_program_data_account_info.data.borrow()[data_offset..];
        let test_program_data = &test_program_data_account_info.data.borrow()[data_offset..];


        if real_program_data.len() < data_offset {
            msg!("Account is too small to be a program data account");
            return Err(ProgramError::InvalidAccountData); 
        }

        if test_program_data.len() < data_offset {
            msg!("Account is too small to be a program data account");
            return Err(ProgramError::InvalidAccountData); 
        }


        // check if the program is upgradeable

        let real_meta : UpgradeableLoaderState = bincode::deserialize_from(&real_program_data_account_info.data.borrow()[..data_offset]).unwrap();

        msg!("data_buffer {:?}", real_meta);

        let mut upgrade_authority = None;
        let mut upgradeable : bool = false;
        match real_meta {
            UpgradeableLoaderState::ProgramData{slot, upgrade_authority_address} => upgrade_authority = upgrade_authority_address,
            _ => println!("Account not upgradeable"),
        }
    
        if upgrade_authority.is_some() {
            upgradeable = true;
        }

        let mut current_state = ProgramMetaData::try_from_slice(&program_metadata_account_info.data.borrow()[..])?;

        if !metadata.verified {
            current_state.verified_code = 1;
        }

        if metadata.verified && upgradeable {
            current_state.verified_code  = 2;
        }

        if metadata.verified && !upgradeable {
            current_state.verified_code  = 3;
        }

        current_state.test_address = metadata.test_address;
        current_state.last_verified_slot = metadata.verified_slot;
        current_state.data_hash = metadata.data_hash;

        msg!("have hash {:?}", metadata.data_hash);
        
        current_state.serialize(&mut &mut program_metadata_account_info.data.borrow_mut()[..])?;


        Ok(())


    }

}