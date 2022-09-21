use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    pubkey::Pubkey,
};



#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Default)]
pub struct ProgramMetaData {
    pub test_address : Pubkey,
    pub last_verified_slot : u64,
    pub verified_code : u8,
    pub data_hash : [u8 ; 32]
}

pub fn get_metadata_size() -> usize {
    let encoded = ProgramMetaData {..Default::default()}
        .try_to_vec().unwrap();

    encoded.len()
}