use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke_signed},
    pubkey::Pubkey, rent,
    native_token::LAMPORTS_PER_SOL
};

pub fn create_program_data_account<'a>(
    funding_account: &AccountInfo<'a>,
    data_account: &AccountInfo<'a>,
    program_id :  &Pubkey,
    bump_seed : u8,
    seed : &[u8],
    data_size : usize
) -> ProgramResult
{

    // Check if the account has already been initialized
    if **data_account.try_borrow_lamports()? > 0 {
        msg!("user's data account is already initialized. skipping");
        return Ok(());
    }

    msg!("Creating user's data account");
        
    // the bidders data account just holds a single usize giving their location in the
    // bid array and a bool
    let space : u64 = data_size.try_into().unwrap();
    let lamports = rent::Rent::default().minimum_balance(data_size);

    msg!("Require {} lamports for {} size data", lamports, data_size);
    let ix = solana_program::system_instruction::create_account(
        funding_account.key,
        data_account.key,
        lamports,
        space,
        program_id,
    );

    // Sign and submit transaction
    invoke_signed(
        &ix,
        &[funding_account.clone(), data_account.clone()],
        &[&[seed, &[bump_seed]]]
    )?;

    Ok(())
}



pub fn to_sol(value : u64) -> f64 {
    (value as f64) / (LAMPORTS_PER_SOL as f64)
}