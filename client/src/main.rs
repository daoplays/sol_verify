
pub mod state;

use crate::state::{Result, SubmitProgramMeta, VerifyInstruction, VerifyProgramMeta, ProgramMetaData, StatusMeta, Network, ProgramJsonData};

use std::borrow::Borrow;
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
use solana_security_txt::security_txt;
use std::fs::File;
use std::io::prelude::*;
use serde::{Deserialize, Serialize};

// some globals
const PROGRAM_KEY : &str = "CNd6wN4en9Xvbf2e1ubb2YyCkC7J1BbbuhAGhqcdHFbi";

const SOLANA_TEST: &str = "https://api.testnet.solana.com";
const SOLANA_DEV: &str = "https://api.devnet.solana.com";
const SOLANA_MAIN: &str = "https://api.mainnet-beta.solana.com";


const URL: &str = SOLANA_DEV;

fn u8_to_network(index :  u8) ->  Network 
{
    match index {
        0 => Network::TestNet,
        1 => Network::DevNet,
        2 => Network::MainNet,
        _ => Network::Invalid
    }
}

pub fn network_to_string(network :  Network) ->  String 
{
    match network {
        Network::TestNet => "test_net".to_string(),
        Network::DevNet => "dev_net".to_string(),
        Network::MainNet => "main_net".to_string(),
        Network::Invalid => "invalid".to_string()

    }
}

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
        let real_address = &args[4];
        let network_arg = &args[5];
        let network_u8: u8 = network_arg.parse().unwrap();
        let network = u8_to_network(network_u8);
        let user_address = &args[6];
        let git_repo = &args[7];
        let git_commit = &args[8];
        let directory = &args[9];

        if network == Network::Invalid {
            println!("invalid network");
            std::process::exit(1);
        }

        if let Err(err) = verify_program(key_file, test_key_file, real_address, network, user_address, git_repo, git_commit, directory) {
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

    if function == "update_status" {

        let user_address = &args[3];
        let status_code_string = &args[4];
        let log_message = &args[5];

        let status_code : u8 = status_code_string.parse().unwrap();


        if let Err(err) = update_status(key_file, user_address, status_code, log_message) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }

    if function == "write_security" {

        let program_address = &args[3];
        if let Err(err) = get_security(program_address) {
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

    println!("real: {:?}", real_result);
    println!("test: {:?}", test_result);

    let mut hashed_array : [u8; 32] = [0; 32];
    for i in 0..32 {
        let hash_slice = &test_result[i..(i+1)];
        hashed_array[i] = u8::from_le_bytes(hash_slice.try_into().expect("slice with incorrect length"));
    }

    
    return (real_result == test_result, hashed_array);
    
}


fn submit_program(key_file: &String) ->Result<()> {

    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let client = RpcClient::new(URL);

    let real_address = Pubkey::from_str("6Eur9ULgxmgcK2Lk8vdCyasBLmU5HzYgmCxsYunr7U52").unwrap();
    let network = Network::DevNet;
    let git_repo = "https://github.com/daoplays/sol_verify/archive/refs/heads/main.zip".to_string();
    let git_commit = "".to_string();
    let directory = "sol_verify-main/program".to_string();
    let docker_version = "".to_string();
    let rust_version = "1.62".to_string();
    let solana_version = "1.10.39".to_string();
    let anchor_version = "0.25.0".to_string();



    let network_string = "dev_net".to_string();

    let program_address = Pubkey::from_str(PROGRAM_KEY).unwrap();

    let (expected_metadata_key, _bump_seed) = Pubkey::find_program_address(&[&real_address.to_bytes(), &network_string.as_bytes()], &program_address);

    let (expected_userdata_key, _bump_seed) = Pubkey::find_program_address(&[&wallet.pubkey().to_bytes(), b"user_account"], &program_address);

    let meta_data =  SubmitProgramMeta{
        address: real_address, 
        network : network,
        git_repo : git_repo, 
        git_commit : git_commit, 
        directory : directory, 
        docker_version : docker_version,
        rust_version : rust_version,
        solana_version : solana_version,
        anchor_version : anchor_version
    };


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

fn get_program_client(network : Network) -> RpcClient {

    match network {
        Network::TestNet => RpcClient::new(SOLANA_TEST),
        Network::DevNet => RpcClient::new(SOLANA_DEV),
        Network::MainNet => RpcClient::new(SOLANA_MAIN),
        Network::Invalid => RpcClient::new(SOLANA_TEST),
    }
}

struct upgrade_info {
    slot : u64,
    upgrade_authority : Option<Pubkey>
}

fn verify_program(key_file: &String, test_key_file: &String, real_address_string: &String, network : Network, user_address : &String, git_repo: &String, git_commit: &String, directory: &String) ->Result<()> {

    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();
    let test_keypair = read_keypair_file(test_key_file).unwrap();


    // (3) Create RPC client to be used to talk to Solana cluster
    let client = RpcClient::new(URL);

    let program_client = get_program_client(network);
/*
   
    let security_txt = solana_security_txt::find_and_parse(program_data).unwrap();
    println!("{}", security_txt);

*/


    let real_address = Pubkey::from_str(real_address_string).unwrap();
    let test_address = test_keypair.pubkey();
    let program_address = Pubkey::from_str(PROGRAM_KEY).unwrap();

    let real_program_account = program_client.get_account(&real_address)?;
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

    let real_program_data_account = program_client.get_account(&real_program_data_address)?;
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


    let real_meta : UpgradeableLoaderState = bincode::deserialize_from(&real_program_data_account.data[..data_offset]).unwrap();

    let mut upgrade_info = upgrade_info {slot : 0, upgrade_authority : Some(system_program::ID)};

    println!("data_buffer {:?}", real_meta);

    let mut upgradeable : bool = false;
    match real_meta {
        UpgradeableLoaderState::ProgramData{slot, upgrade_authority_address} => 
        upgrade_info = upgrade_info {slot : slot, upgrade_authority  : upgrade_authority_address},
        _ => println!("Account not upgradeable"),
    }

    
    if upgrade_info.upgrade_authority.is_some() {
        upgradeable = true;
    }

    let mut verified_code : u8 = 1;
    if !verified {
        verified_code = 1;
    }

    if verified && upgradeable {
        verified_code  = 2;
    }

    if verified && !upgradeable {
        verified_code  = 3;
    }

    let current_slot = program_client.get_slot()?;

    let network_string = network_to_string(network);
    let (expected_metadata_key, _bump_seed) = Pubkey::find_program_address(&[&real_address.to_bytes(), &network_string.as_bytes()], &program_address);

    // check the current meta data
    let response = client.get_account_data(&expected_metadata_key)?;
    let current_state = ProgramMetaData::try_from_slice(&response[..]).unwrap();

    let mut message = "Program ".to_owned();
    message += real_address_string;
    message += " : ";

    // if the program has already had a verification run, and it has code = 3 (passed, immutable) then we only want to update
    // the state if it still passes

    if current_state.verified_code == 3 && !verified {
        message += "program already verified and immutable but new verification fails.  Not updating state.";
        update_status(key_file, user_address, 109  as u8, &message)?;
        return Ok(println!("{}", message))
    }

    // if a previous verification was code = 2 (passed, mutable), then we need to check if the last update time was after the last verification time.  If it wasn't then we only update if the verification still passes.

    if current_state.verified_code == 2 && upgrade_info.slot < current_state.last_verified_slot && !verified {
        message +=  "program already verified and not updated since last verification but new verification fails.  Not updating state.";
        update_status(key_file, user_address, 110  as u8, &message)?;
        return Ok(println!("{}", message))
    }

    // if the update time is after the last verification time then we will always perform the verification again.
    // Similarly if code == 1 or 0 (failed, or this is the first time) then we will always perform it again.

    let meta_data =  VerifyProgramMeta{verified_code: verified_code, real_address : real_address, test_address : test_address, data_hash : test_hash, verified_slot : current_slot, network : network, git_repo : git_repo.to_string(), git_commit: git_commit.to_string(), directory : directory.to_string() };

    let instruction = Instruction::new_with_borsh(
        program_address,
        &VerifyInstruction::VerifyProgram {metadata : meta_data},
        vec![
            AccountMeta::new_readonly(wallet.pubkey(), true),
            AccountMeta::new(expected_metadata_key, false),

            AccountMeta::new_readonly(real_address, false),
            AccountMeta::new_readonly(test_address, false)
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

 

    // finally update the user with the result
    if verified_code == 1 {
        message += "Verification process has not produced a match";
        update_status(key_file, user_address, 1  as u8, &message)?;
        return Ok(println!("{}", message))
    }

    if verified_code == 2 {
        message +=  "Verification was successful, however the program is upgradable";
        update_status(key_file, user_address, 1  as u8, &message)?;
        return Ok(println!("{}", message))
    }

    if verified_code == 3 {
        message +=  "Verification was successful, and program is immutable";
        update_status(key_file, user_address, 1  as u8, &message)?;
        return Ok(println!("{}", message))
    }

    Ok(println!("Success!"))
}


fn check_metadata() ->Result<()> {

 
    // (3) Create RPC client to be used to talk to Solana cluster
    let client = RpcClient::new(URL);


    let real_address = Pubkey::from_str("7EGMFCt38NyXZHsR7G3JeBgMkNPhGF3z8g1pVLEXPA8Y").unwrap();
    let program_address = Pubkey::from_str(PROGRAM_KEY).unwrap();

    let network_string : String = "dev_net".to_string();
    let (expected_metadata_key, _bump_seed) = Pubkey::find_program_address(&[&real_address.to_bytes(), &network_string.as_bytes()], &program_address);

    let response = client.get_account_data(&expected_metadata_key)?;
    println!("data in account: {}", expected_metadata_key);

    let current_state = ProgramMetaData::try_from_slice(&response[..]).unwrap();

    println!("verified: {}", current_state.verified_code);   
    println!("last_verified_slot: {}", current_state.last_verified_slot);    
    println!("test_address: {}", current_state.test_address); 
    println!("data: {:?}", current_state.data_hash);    
 

    Ok(println!("Success!"))
}


fn update_status(key_file : &String, user_address : &String, status_code : u8, log_message : &String) ->Result<()> {

    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let client = RpcClient::new(URL);

    let program_address = Pubkey::from_str(PROGRAM_KEY).unwrap();
    let user_pubkey = Pubkey::from_str(user_address).unwrap();


    let (expected_userdata_key, _bump_seed) = Pubkey::find_program_address(&[&user_pubkey.to_bytes(), b"user_account"], &program_address);

    let meta_data =  StatusMeta{user_pubkey : user_pubkey, status_code : status_code, log_message : log_message.to_string()};


    let instruction = Instruction::new_with_borsh(
        program_address,
        &VerifyInstruction::UpdateStatus {metadata : meta_data},
        vec![
            AccountMeta::new_readonly(wallet.pubkey(), true),
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

    let signature = client.send_transaction(&txn)?;
    println!("signature: {}", signature);
    

    Ok(println!("Success!"))
}


fn get_security(program_address_string : &String) ->Result<()> {

    let client = RpcClient::new(URL);

    let program_address = Pubkey::from_str(program_address_string).unwrap();

    let program_account = client.get_account(&program_address)?;

    if !bpf_loader_upgradeable::check_id(&program_account.owner) {
        println!("wrong owner");
        return Ok(());
    }

    let program: UpgradeableLoaderState = bincode::deserialize_from(&program_account.data[..]).unwrap();

    let program_data_address = if let UpgradeableLoaderState::Program {
        programdata_address,
    } = program
    {
        programdata_address
    } else {
        return Ok(());
    };

    let program_data_account = client.get_account(&program_data_address)?;

    let offset = UpgradeableLoaderState::programdata_data_offset().unwrap();
    if program_data_account.data.len() < offset {
        return Ok(());
    }

    let real_meta : UpgradeableLoaderState = bincode::deserialize_from(&program_data_account.data[..offset]).unwrap();

    let mut upgrade_info = upgrade_info {slot : 0, upgrade_authority : Some(system_program::ID)};

    println!("data_buffer {:?}", real_meta);

    let mut upgradeable : bool = false;
    match real_meta {
        UpgradeableLoaderState::ProgramData{slot, upgrade_authority_address} => 
        upgrade_info = upgrade_info {slot : slot, upgrade_authority  : upgrade_authority_address},
        _ => println!("Account not upgradeable"),
    }

    if upgrade_info.upgrade_authority.is_some() {
        upgradeable = true;
    }

    let program_data = &program_data_account.data[offset..];

    let security_txt = solana_security_txt::find_and_parse(program_data).unwrap();
    println!("{}", security_txt);

    let mut json_data = ProgramJsonData {
        upgradeable: upgradeable,
        source_code: "none".to_owned(),
    };


    if security_txt.source_code.is_some() {
        json_data.source_code = security_txt.source_code.unwrap();
    }

    let j = serde_json::to_string(&json_data).unwrap();

    let mut file = File::create("program_meta_data.json").unwrap();
    file.write_all(j.as_bytes()).unwrap();


    Ok(())

}
