mod program_option;

use std::{io, process};
use anyhow::{Result};
use clap::Parser;
use solana_sdk::signature::{Keypair, Signature};
use solana_transaction_status_client_types::EncodedConfirmedTransactionWithStatusMeta;
use {
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_client::rpc_config::RpcTransactionConfig,
    solana_sdk::{
        pubkey::Pubkey, instruction::{AccountMeta, Instruction},
        commitment_config::{CommitmentConfig, CommitmentLevel},
        signature::{read_keypair_file, Signer},
        transaction::Transaction,
    },
    solana_transaction_status_client_types::UiTransactionEncoding,
    std::{
        str::FromStr, path::Path
    },
};
use crate::program_option::Args;

// path to keypair
const KEYPAIR_PATH: &str = "/home/kvasshtain/.config/solana/id.json";
// solana is running on local machine
const SOLANA_URL: &str = "http://localhost:8899";
// program_id of the "memo" program
// https://github.com/solana-program/memo/blob/37568de8dae6a4e69572a85e8c166910da232b90/program/src/lib.rs#L26
const PROGRAM_ID: &str = "4fnvoc7wADwtwJ9SRUvL7KpCBTp8qztm5GqjZBFP7GTt";
//System Program account pubkey
const SYSTEM_PRG_PUBKEY: &str = "11111111111111111111111111111111";


pub fn ask_mode_and_close_if_not_correct() -> u8 {
    let prg_modes = [0, 1];

    let mut program_mode_input = String::new();
    println!("Please select mode (0 - Create account, 1 - Resize account): ");
    let reading_result = io::stdin().read_line(&mut program_mode_input);

    match reading_result{
        Ok(_size) => {},
        Err(_err_message) => {
            println!("Mode isn't supported");
            process::exit(0);
        }
    }

    let program_mode_str = program_mode_input.trim_end();
    let parsing_result: Result<u8, _> = program_mode_str.parse();
    let mode: u8;

    match parsing_result{
        Ok(parsed_mode) => { mode = parsed_mode; },
        Err(_err_message) => {
            println!("Mode isn't supported");
            process::exit(0);
        }
    }

    match mode {
        x if prg_modes.contains(&x) => {},
        _ => {
            println!("Mode isn't supported");
            process::exit(0);
        },
    }

    mode
}

pub fn ask_seed_and_close_if_not_correct() -> Vec<u8> {
    let mut seed_input: String = String::new();
    println!("Please enter seed: ");
    let reading_result = io::stdin().read_line(&mut seed_input);

    match reading_result{
        Ok(_size) => {},
        Err(_err_message) => {
            println!("Seed is incorrect");
            process::exit(0);
        }
    }

    Vec::from(seed_input.trim_end().as_bytes())
}

pub fn ask_new_size_and_close_if_not_correct() -> u64 {
    let mut account_new_size_input = String::new();
    println!("Please enter new account size: ");
    let reading_result = io::stdin().read_line(&mut account_new_size_input);

    match reading_result{
        Ok(_size) => {},
        Err(_err_message) => {
            println!("Account size is incorrect");
            process::exit(0);
        }
    }

    let parsing_result: Result<u64, _> = account_new_size_input.trim_end().parse();

    let new_size: u64;

    match parsing_result{
        Ok(parsed_new_size) => { new_size = parsed_new_size; },
        Err(error) => {
            println!("Size is incorrect: {}", error);
            process::exit(0);
        }
    }

    new_size
}

// create instuction of the memo-program
pub fn build_instruction(data: &[u8], tx_sig_pubkey: Pubkey, new_pda_key: Pubkey, sys_prg_pubkey: Pubkey) -> Instruction {
    Instruction {
        program_id: Pubkey::from_str(PROGRAM_ID).unwrap(),
        accounts: Vec::from([
            AccountMeta::new(tx_sig_pubkey, true),
            AccountMeta::new(new_pda_key, false),
            AccountMeta::new_readonly(sys_prg_pubkey, false)]),//accounts_pubkeys
        data: data.to_vec(),
    }
}

pub async fn send_instruction(data: Vec<u8>, client: &RpcClient, tx_sig: Keypair, new_pda_key: Pubkey, sys_prg_pubkey: Pubkey) -> Result<Signature> {
    let data_slice = data.as_slice();
    let tx_sig_pubkey = tx_sig.pubkey();
    // create instruction
    let ix = build_instruction(&data_slice, tx_sig_pubkey, new_pda_key, sys_prg_pubkey);

    // take a look at purpose of the blockhash:
    // https://solana.com/docs/core/transactions#recent-blockhash
    let blockhash = client.get_latest_blockhash().await?;

    // solana tx
    let mut tx =
        Transaction::new_with_payer(&[ix], Some(&tx_sig_pubkey));
    tx.sign(&[&tx_sig], blockhash);

    // let's send it!
    Ok(client.send_and_confirm_transaction(&tx).await?)
}

pub async fn read_transaction(client: &RpcClient, sig: Signature) -> Result<EncodedConfirmedTransactionWithStatusMeta> {
    let config = RpcTransactionConfig {
        commitment: CommitmentConfig::confirmed().into(),
        encoding: UiTransactionEncoding::Base64.into(),
        max_supported_transaction_version: Some(0),
    };

    Ok(client.get_transaction_with_config(&sig, config).await?)
}

#[tokio::main]
async fn main() -> Result<()> {

    let args = Args::parse();

    let program_mode: u8;

    if args.mode.is_none() {
        program_mode = ask_mode_and_close_if_not_correct();
    }
    else {
        program_mode = args.mode.unwrap();
    }

    let seed: Vec<u8>;

    if args.seed.is_none() {
        seed = ask_seed_and_close_if_not_correct();
    }
    else {
        seed = Vec::from(args.seed.unwrap().as_bytes());
    }

    let mut new_size = 0u64;

    match program_mode {
        0 => {},
        1 => {
            if args.size.is_none() {
                new_size = ask_new_size_and_close_if_not_correct();
            }
            else {
                new_size = args.size.unwrap();
            }
        },
        _ => {
            println!("Mode isn't supported");
            process::exit(0);
        },
    }

    
    
    // 1: first account - transaction signer
    let tx_sig =  read_keypair_file(Path::new(KEYPAIR_PATH)).unwrap();

    // 2: 2nd - account to create
    let program_id: &Pubkey = &Pubkey::from_str(PROGRAM_ID)?;
    let (new_pda_key, _bump) = Pubkey::find_program_address(&[&*seed], &program_id);

    // 3: system_program account
    let sys_prg_pubkey = Pubkey::from_str(SYSTEM_PRG_PUBKEY)?;

    
    
    //rpc-client, it will be used to send transaction to solana-validator
    let client = RpcClient::new_with_commitment(
        SOLANA_URL.to_string(),
        CommitmentConfig {
            commitment: CommitmentLevel::Confirmed,
        },
    );


    let mut data = vec![program_mode];

    match program_mode {
        0 => {
            data.extend(seed);
        },
        1 => {
            data.extend(new_size.to_le_bytes());
        },
        _ => {
            println!("Mode isn't supported");
            process::exit(0);
        },
    }

    
    
    // let's send it!
    let  sig= send_instruction(data, &client, tx_sig, new_pda_key, sys_prg_pubkey).await?;

    println!("we have done it, solana signature: {}", sig);

    let transaction = read_transaction(&client, sig).await?;

    println!("Transaction data is {:#?}", transaction);

    Ok(())
}