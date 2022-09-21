use solana_program::{program_error::ProgramError, pubkey::Pubkey};
use borsh::{BorshDeserialize, BorshSerialize};
use crate::error::DaoPlaysError::InvalidInstruction;


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


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum VerifyInstruction {

    SubmitProgram {
        metadata: SubmitProgramMeta
    },
    VerifyProgram {
        metadata : VerifyProgramMeta
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
            _ => return Err(InvalidInstruction.into()),
        })
    }
}