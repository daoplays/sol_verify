
pub mod state;

use crate::state::{Result, SubmitProgramMeta, VerifyInstruction, VerifyProgramMeta, ProgramMetaData};

use std::{env, io::BufRead};
use std::str::FromStr;
use solana_client::rpc_client::RpcClient;
use solana_program::{pubkey::Pubkey, rent, native_token::LAMPORTS_PER_SOL, system_program};
use solana_sdk::{
    signer::Signer,
    instruction::{AccountMeta, Instruction},
    transaction::Transaction, signer::keypair::read_keypair_file, hash,
    bpf_loader_upgradeable::{self, UpgradeableLoaderState}
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_transaction_status::UiTransactionEncoding;
use twoway::find_bytes;
use sha2::{Sha256, Digest};


// some globals
const PROGRAM_KEY : &str = "4xTTRRsDAjme4JoZxQ87czQvmstZ6onJJdNAQXpPw9PA";

const SOLANA_DEV: &str = "https://api.devnet.solana.com";

const URL: &str = SOLANA_DEV;

fn main() {
    let args: Vec<String> = env::args().collect();
    let key_file = &args[1];
    let function = &args[2];

    if function == "submit" {

        if let Err(err) = submit_program(key_file) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }

    if function == "verify" {

        let test_key_file = &args[3];
        if let Err(err) = verify_program(key_file, test_key_file) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }

    if function == "check_metadata" {

        if let Err(err) = check_metadata() {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }



}

// create a sha256 hash from our initial seed and a nonce value to produce 4 64bit random numbers
fn get_sha256_hashed_data(real_data : &[u8], test_data: &[u8]) -> (bool, [u8; 32]) {

    let mut real_hasher = Sha256::new();
    real_hasher.update(real_data);

    let real_result = real_hasher.finalize();

    let mut test_hasher = Sha256::new();
    test_hasher.update(test_data);

    let test_result = test_hasher.finalize();

    println!("test: {:?}", test_result);

    let mut hashed_array : [u8; 32] = [0; 32];
    for i in 0..32 {
        let hash_slice = &test_result[i..(i+1)];
        hashed_array[i] = u8::from_le_bytes(hash_slice.try_into().expect("slice with incorrect length"));
    }

    println!("test: {:?}", hashed_array);
    
    return (real_result == test_result, hashed_array);
    
}


fn submit_program(key_file: &String) ->Result<()> {

    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let client = RpcClient::new(URL);

    let real_address = Pubkey::from_str("7EGMFCt38NyXZHsR7G3JeBgMkNPhGF3z8g1pVLEXPA8Y").unwrap();
    let git_repo = "https://github.com/daoplays/solana_examples.git".to_string();
    let git_commit = "f3dd81928e49299f04070dfc58dd5cd3dd48a682".to_string();
    let directory = "charity_auction/program".to_string();
    let docker_version = "solana_v1.1.14".to_string();

    let program_address = Pubkey::from_str(PROGRAM_KEY).unwrap();

    let (expected_metadata_key, _bump_seed) = Pubkey::find_program_address(&[&real_address.to_bytes()], &program_address);

    let (expected_userdata_key, _bump_seed) = Pubkey::find_program_address(&[&wallet.pubkey().to_bytes()], &program_address);

    let meta_data =  SubmitProgramMeta{address: real_address, git_repo : git_repo, git_commit : git_commit, directory : directory, docker_version : docker_version };


    let instruction = Instruction::new_with_borsh(
        program_address,
        &VerifyInstruction::SubmitProgram {metadata : meta_data},
        vec![
            AccountMeta::new_readonly(wallet.pubkey(), true),
            AccountMeta::new(expected_metadata_key, false),
            AccountMeta::new(expected_userdata_key, false),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false)


        ],
    );

    let signers = [&wallet];
    let instructions = vec![instruction];
    let recent_hash = client.get_latest_blockhash()?;

    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&wallet.pubkey()),
        &signers,
        recent_hash,
    );

    let signature = client.send_and_confirm_transaction(&txn)?;
    println!("signature: {}", signature);
    let response = client.get_transaction(&signature, UiTransactionEncoding::Json)?;
    println!("result: {:#?}", response); 

    Ok(println!("Success!"))
}


fn verify_program(key_file: &String, test_key_file: &String) ->Result<()> {

    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();
    let test_keypair = read_keypair_file(test_key_file).unwrap();


    // (3) Create RPC client to be used to talk to Solana cluster
    let client = RpcClient::new(URL);
/*
   
    let security_txt = solana_security_txt::find_and_parse(program_data).unwrap();
    println!("{}", security_txt);

*/


    let real_address = Pubkey::from_str("7EGMFCt38NyXZHsR7G3JeBgMkNPhGF3z8g1pVLEXPA8Y").unwrap();
    let test_address = test_keypair.pubkey();
    let program_address = Pubkey::from_str(PROGRAM_KEY).unwrap();

    let real_program_account = client.get_account(&real_address)?;
    let test_program_account = client.get_account(&test_address)?;

    if !bpf_loader_upgradeable::check_id(&real_program_account.owner) {
        println!("Only accounts owned by the bpf_loader_upgradeable program are supported at the moment");
        return Ok(());
    }

    if !bpf_loader_upgradeable::check_id(&test_program_account.owner) {
        println!("Only accounts owned by the bpf_loader_upgradeable program are supported at the moment");
        return Ok(());
    }

    let real_program: UpgradeableLoaderState = bincode::deserialize_from(&real_program_account.data[..]).unwrap();
    let test_program: UpgradeableLoaderState = bincode::deserialize_from(&test_program_account.data[..]).unwrap();

    let real_program_data_address = if let UpgradeableLoaderState::Program {
        programdata_address,
    } = real_program
    {
        programdata_address
    } else {
        println!("Wrong program account type");
        return Ok(());
    };

    let test_program_data_address = if let UpgradeableLoaderState::Program {
        programdata_address,
    } = test_program
    {
        programdata_address
    } else {
        println!("Wrong program account type");
        return Ok(());
    };

    let real_program_data_account = client.get_account(&real_program_data_address)?;
    let test_program_data_account = client.get_account(&test_program_data_address)?;

    let data_offset = UpgradeableLoaderState::programdata_data_offset().unwrap();
    if real_program_data_account.data.len() < data_offset {
        println!("Real account is too small to be a program data account");
        return Ok(());
    }

    if test_program_data_account.data.len() < data_offset {
        println!("Test account is too small to be a program data account");
        return Ok(());
    }

    let real_program_data = &real_program_data_account.data[data_offset..];
    let test_program_data = &test_program_data_account.data[data_offset..];


    let (verified, test_hash) = get_sha256_hashed_data(real_program_data, test_program_data);

    println!("verified {}", verified);

    let (expected_metadata_key, _bump_seed) = Pubkey::find_program_address(&[&real_address.to_bytes()], &program_address);

    let current_slot = client.get_slot()?;

    let meta_data =  VerifyProgramMeta{verified: verified, real_address : real_address, test_address : test_address, data_hash : test_hash, verified_slot : current_slot };


    let instruction = Instruction::new_with_borsh(
        program_address,
        &VerifyInstruction::VerifyProgram {metadata : meta_data},
        vec![
            AccountMeta::new_readonly(wallet.pubkey(), true),
            AccountMeta::new(expected_metadata_key, false),

            AccountMeta::new_readonly(real_address, false),
            AccountMeta::new_readonly(test_address, false),

            AccountMeta::new_readonly(real_program_data_address, false),
            AccountMeta::new_readonly(test_program_data_address, false)
        ],
    );

    let signers = [&wallet];
    let instructions = vec![instruction];
    let recent_hash = client.get_latest_blockhash()?;

    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&wallet.pubkey()),
        &signers,
        recent_hash,
    );

    let signature = client.send_and_confirm_transaction(&txn)?;
    println!("signature: {}", signature);
    let response = client.get_transaction(&signature, UiTransactionEncoding::Json)?;
    println!("result: {:#?}", response); 

    Ok(println!("Success!"))
}


fn check_metadata() ->Result<()> {

 
    // (3) Create RPC client to be used to talk to Solana cluster
    let client = RpcClient::new(URL);


    let real_address = Pubkey::from_str("7EGMFCt38NyXZHsR7G3JeBgMkNPhGF3z8g1pVLEXPA8Y").unwrap();
    let program_address = Pubkey::from_str(PROGRAM_KEY).unwrap();

    let (expected_metadata_key, _bump_seed) = Pubkey::find_program_address(&[&real_address.to_bytes()], &program_address);

    let response = client.get_account_data(&expected_metadata_key)?;
    println!("data in account: {}", expected_metadata_key);

    let current_state = ProgramMetaData::try_from_slice(&response[..]).unwrap();

    println!("verified: {}", current_state.verified_code);   
    println!("last_verified_slot: {}", current_state.last_verified_slot);    
    println!("test_address: {}", current_state.test_address); 
    println!("data: {:?}", current_state.data_hash);    
 

    Ok(println!("Success!"))
}

