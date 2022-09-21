use solana_program::{pubkey::Pubkey, declare_id};
// functions to calculate expected public keys

// This can also be replaced with pubkey ("CU8AequXiVdXyVKc7Vqg2jiBDJgPwapMbcBrm7EVnTtm") if you are on a recent sdk
mod daoplays {
    use super::*;
    declare_id!("FxVpjJ5AGY6cfCwZQP5v8QBfS4J2NPa62HbGh1Fu2LpD");   
}


pub fn get_expected_daoplays_key() -> Pubkey
{
    daoplays::ID
}
