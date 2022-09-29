use std::str;
use crate::instruction;
use crate::state::{ProgramMetaData};
use borsh::{BorshDeserialize, BorshSerialize};
use crate::accounts;
use crate::utils;
use crate::state;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    msg,
    pubkey::Pubkey
};

use crate::{instruction::{VerifyInstruction, SubmitProgramMeta, VerifyProgramMeta, StatusMeta}};

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
            },
            VerifyInstruction::UpdateStatus {metadata} => {
                msg!("Instruction: Update Status");
                Self::update_status(accounts, program_id, metadata)
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
        let user_metadata_account_info = next_account_info(account_info_iter)?;

        let system_program_account_info = next_account_info(account_info_iter)?;

        msg!("in submit, check signer");
        // the first account should be the funding account and should be a signer
        if !program_owner_account_info.is_signer {
            msg!("expected first account as signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        let network_id : String = instruction::network_to_string(metadata.network);
        let (expected_metadata_key, bump_seed) = Pubkey::find_program_address(&[&metadata.address.to_bytes(), &network_id.as_bytes()], &program_id);
        
        msg!("in submit, check program meta data");
        if program_metadata_account_info.key != &expected_metadata_key
        { 
            msg!("expected second account to be the program metadata account {}", expected_metadata_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        let (expected_user_metadata_key, user_bump_seed) = Pubkey::find_program_address(&[&program_owner_account_info.key.to_bytes(), b"user_account"], &program_id);

        msg!("in submit, check user meta data");
        if user_metadata_account_info.key != &expected_user_metadata_key
        { 
            msg!("expected third account to be the user metadata account {}", expected_user_metadata_key);
            return Err(ProgramError::InvalidAccountData); 
        }
        
        msg!("in submit, check system program");
        // the third and final account is the system_program
        if system_program_account_info.key != &solana_program::system_program::id() {
            msg!("expected fourth account to be the system program {}", solana_program::system_program::id());
            return Err(ProgramError::InvalidAccountData);
        }
        
        // create the program meta data account if we need it
        utils::create_program_data_account(
            program_owner_account_info,
            program_metadata_account_info,
            program_id,
            bump_seed,
            &metadata.address.to_bytes(),
            network_id.as_bytes(),
            state::get_metadata_size()
        )?;

        // create the user meta data account if we need it
        utils::create_program_data_account(
            program_owner_account_info,
            user_metadata_account_info,
            program_id,
            user_bump_seed,
            &program_owner_account_info.key.to_bytes(),
            b"user_account",
            state::get_userdata_size()
        )?;

        let log_array = ["Program", &metadata.address.to_string(), ": accounts created"];
        let log_message = (log_array.join(" ")).to_string();
        let log_message_bytes = log_message.as_bytes();

        let s = match str::from_utf8(log_message_bytes) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };

        msg!("try string again {}", s);

        let message_len = log_message_bytes.len() as u8;
        let end_index = 2 + message_len as usize;
        message_len.serialize(&mut &mut user_metadata_account_info.data.borrow_mut()[1..2])?;

        msg!("serialize bytes size {} end {} max {}", message_len, end_index, state::get_userdata_size());
        for i  in 0..message_len {
            let index = i as usize;
            let start = 2 + i as usize;
            let end = start + 1 as usize;
            log_message_bytes[index].serialize(&mut &mut user_metadata_account_info.data.borrow_mut()[start..end])?;

        }


        Ok(())

    }


    fn update_status(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        metadata : StatusMeta
    ) ->ProgramResult 
    {

        let account_info_iter = &mut accounts.iter();

        // This function expects to be passed eight accounts, get them all first and then check their value is as expected
        let dao_plays_account_info = next_account_info(account_info_iter)?;
        let user_metadata_account_info = next_account_info(account_info_iter)?;

        let system_program_account_info = next_account_info(account_info_iter)?;

        // the first account should be the funding account and should be a signer
        if !dao_plays_account_info.is_signer {
            msg!("expected first account as signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        // the first account is the daoplays SOL address
        if dao_plays_account_info.key != &accounts::get_expected_daoplays_key()
        {
            msg!("expected first account to be the verifier address {}", accounts::get_expected_daoplays_key());
            return Err(ProgramError::InvalidAccountData);
        }

        let (expected_user_metadata_key, _user_bump_seed) = Pubkey::find_program_address(&[&metadata.user_pubkey.to_bytes(), b"user_account"], &program_id);
        
        if user_metadata_account_info.key != &expected_user_metadata_key
        { 
            msg!("expected second account to be the user metadata account {}", expected_user_metadata_key);
            return Err(ProgramError::InvalidAccountData); 
        }
        
        // check that the user id account exists
        if **user_metadata_account_info.try_borrow_lamports()? == 0 {
            msg!("user's meta data account doesn't exist");
            return Err(ProgramError::InvalidAccountData);
        }

        // the third and final account is the system_program
        if system_program_account_info.key != &solana_program::system_program::id() {
            msg!("expected third account to be the system program {}", solana_program::system_program::id());
            return Err(ProgramError::InvalidAccountData);
        }
        
        metadata.status_code.serialize(&mut &mut user_metadata_account_info.data.borrow_mut()[0..1])?;
        let log_message_bytes = metadata.log_message.as_bytes();

        let message_len = log_message_bytes.len() as u8;
        message_len.serialize(&mut &mut user_metadata_account_info.data.borrow_mut()[1..2])?;

        msg!("serialize bytes size {}", message_len);
        for i  in 0..message_len {
            let index = i as usize;
            let start = 2 + i as usize;
            let end = start + 1 as usize;
            log_message_bytes[index].serialize(&mut &mut user_metadata_account_info.data.borrow_mut()[start..end])?;

        }

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

        let network_id : String = instruction::network_to_string(metadata.network);
        let (expected_metadata_key, _bump_seed) = Pubkey::find_program_address(&[&metadata.real_address.to_bytes(), &network_id.as_bytes()], &program_id);
        
        
        if program_metadata_account_info.key != &expected_metadata_key
        { 
            msg!("expected second account to be the program metadata account {}", expected_metadata_key);
            return Err(ProgramError::InvalidAccountData); 
        }


        let mut current_state = ProgramMetaData::try_from_slice(&program_metadata_account_info.data.borrow()[..])?;

        current_state.verified_code = metadata.verified_code;
        current_state.test_address = metadata.test_address;
        current_state.last_verified_slot = metadata.verified_slot;
        current_state.data_hash = metadata.data_hash;

        msg!("current_state {:?}", current_state);
        
        current_state.serialize(&mut &mut program_metadata_account_info.data.borrow_mut()[..])?;


        Ok(())


    }

}