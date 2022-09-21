use thiserror::Error;
use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{pubkey::Pubkey};


#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to read solana config file: ({0})")]
    ConfigReadError(std::io::Error),
   
    #[error("invalid config: ({0})")]
    InvalidConfig(String),

    #[error("serialization error: ({0})")]
    SerializationError(std::io::Error),

    #[error("solana client error: ({0})")]
    ClientError(#[from] solana_client::client_error::ClientError),

    #[error("error in public key derivation: ({0})")]
    KeyDerivationError(#[from] solana_sdk::pubkey::PubkeyError),
}

pub type Result<T> = std::result::Result<T, Error>;


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct SubmitProgramMeta {
    // the amount of supporter tokens to be sent to the user
    pub address : Pubkey,
    pub git_repo : String,
    pub git_commit : String,
    pub directory : String,
    pub docker_version : String
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct VerifyProgramMeta {
    // the amount of supporter tokens to be sent to the user
    pub verified : bool,
    pub real_address : Pubkey,
    pub test_address : Pubkey,
    pub data_hash : [u8; 32],
    pub verified_slot : u64
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Default)]
pub struct ProgramMetaData {
    pub test_address : Pubkey,
    pub last_verified_slot : u64,
    pub verified_code : u8,
    pub data_hash : [u8 ; 32]
}



#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum VerifyInstruction {

    SubmitProgram {
        metadata: SubmitProgramMeta
    },
    VerifyProgram {
        metadata : VerifyProgramMeta
    }
}
