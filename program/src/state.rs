use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    pubkey::Pubkey,
};
use solana_security_txt::security_txt;

pub const GIT_REPO_BEGIN: &str = "=======BEGIN GIT REPO=======\0";
pub const GIT_REPO_END: &str = "=======END GIT REPO=======\0";

pub const GIT_COMMIT_BEGIN: &str = "=======BEGIN GIT COMMIT=======\0";
pub const GIT_COMMIT_END: &str = "=======END GIT COMMIT=======\0";

pub const GIT_DIR_BEGIN: &str = "=======BEGIN GIT DIR=======\0";
pub const GIT_DIR_END: &str = "=======END GIT DIR=======\0";

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct ProgramMetaData {
    pub test_address : Pubkey,
    pub last_verified_slot : u64,
    pub verified_code : u8,
    pub data_hash : [u8 ; 32],
    pub code_meta : [u8 ; 512],
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Default)]
pub struct UserMetaData {
    pub status_code : u8,
    pub string_len : u8,
    pub log_message : String
}

pub fn get_metadata_size() -> usize {
    let encoded = ProgramMetaData {test_address : solana_program::system_program::id(), last_verified_slot : 0, verified_code : 0, data_hash : [0 ; 32], code_meta : [0; 512]}
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


security_txt! {
    // Required fields
    name: "SolVerified",
    project_url: "https://www.daoplays.org/verified",
    contacts: "email:daoplays@outlook.com,link:https://daoplays.org,twitter:dao_plays",
    policy: "A tea or coffee should we ever be in the same place",

    // Optional Fields
    preferred_languages: "en",
    source_code: "https://github.com/daoplays/sol_verify",
    acknowledgements: "solana-dev discord"
}