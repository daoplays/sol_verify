use solana_program::{pubkey::Pubkey, declare_id};
// functions to calculate expected public keys

mod daoplays {
    use super::*;
    declare_id!("7LtYL85tZPpYweZMqeHzX6DAaGsrY61DEtnwiPyJaVCD");   
}


pub fn get_expected_daoplays_key() -> Pubkey
{
    daoplays::ID
}
