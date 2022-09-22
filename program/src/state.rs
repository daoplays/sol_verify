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

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Default)]
pub struct UserMetaData {
    pub status_code : u8,
    pub string_len : u8,
    pub log_message : String
}

pub fn get_metadata_size() -> usize {
    let encoded = ProgramMetaData {..Default::default()}
        .try_to_vec().unwrap();

    encoded.len()
}

// we will allow log messages up to 256 characters
pub fn get_userdata_size() -> usize {
    let size : usize = 2 + 4 * 255;

    return size;
}

pub fn get_current_userdata_size(user_data: UserMetaData) -> usize {

    let encoded = user_data.try_to_vec().unwrap();

    encoded.len()
}