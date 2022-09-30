use solana_program::{program_error::ProgramError, pubkey::Pubkey};
use borsh::{BorshDeserialize, BorshSerialize};
use crate::error::DaoPlaysError::InvalidInstruction;


pub fn network_to_string(network :  Network) ->  String 
{
    match network {
        Network::TestNet => "test_net".to_string(),
        Network::DevNet => "dev_net".to_string(),
        Network::MainNet => "main_net".to_string()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum Network {
    TestNet,
    DevNet,
    MainNet
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct SubmitProgramMeta {
    // the amount of supporter tokens to be sent to the user
    pub address : Pubkey,
    pub network : Network,
    pub git_repo : String,
    pub git_commit : String,
    pub directory : String,
    pub docker_version : String,
    pub rust_version : String,
    pub solana_version : String,
    pub anchor_version : String
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct VerifyProgramMeta {
    // the amount of supporter tokens to be sent to the user
    pub verified_code : u8,
    pub real_address : Pubkey,
    pub test_address : Pubkey,
    pub data_hash : [u8; 32],
    pub verified_slot : u64,
    pub network : Network,
    pub git_repo : String,
    pub git_commit : String,
    pub directory : String
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct StatusMeta {
    pub user_pubkey : Pubkey,
    pub status_code : u8,
    pub log_message : String
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum VerifyInstruction {

    SubmitProgram {
        metadata: SubmitProgramMeta
    },
    VerifyProgram {
        metadata : VerifyProgramMeta
    },
    UpdateStatus {
        metadata : StatusMeta
    }
}

impl VerifyInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction].
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => Self::SubmitProgram  {
                metadata: SubmitProgramMeta::try_from_slice(&rest)?,
            },
            1 => Self::VerifyProgram  {
                metadata: VerifyProgramMeta::try_from_slice(&rest)?,
            },
            2 => Self::UpdateStatus  {
                metadata: StatusMeta::try_from_slice(&rest)?,
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }
}