use std::ptr::null;
use anyhow::Result;
use solana_sdk::precompiles::PrecompileError::InvalidPublicKey;
use solana_sdk::signature::Keypair;
use {
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_client::rpc_config::RpcTransactionConfig,
    solana_sdk::{
        message::Message,
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

// path to keypair
const KEYPAIR_PATH: &str = "/home/kvasshtain/.config/solana/id.json";
// solana is running on local machine
const SOLANA_URL: &str = "http://localhost:8899";
// program_id of the "memo" program
// https://github.com/solana-program/memo/blob/37568de8dae6a4e69572a85e8c166910da232b90/program/src/lib.rs#L26
const PROGRAM_ID: &str = "22Nnm2GqaebkwgFQC95c9ydGp3HZVFbVjrLA5ZBkjKLQ";
//System Program account pubkey
const SYSTEM_PRG_PUBKEY: &str = "11111111111111111111111111111111";

// create instuction of the memo-program
pub fn build_instruction(data: &[u8], transaction_signer_pubkey: Pubkey, account_to_create_pubkey: Pubkey, sys_prg_pubkey: Pubkey) -> Instruction {//accounts_pubkeys: &[&Pubkey]) -> Instruction {
    Instruction {
        program_id: Pubkey::from_str(PROGRAM_ID).unwrap(),
        accounts: Vec::from([
            AccountMeta::new(transaction_signer_pubkey, true),
            AccountMeta::new(account_to_create_pubkey, false),
            AccountMeta::new_readonly(sys_prg_pubkey, false)]),//accounts_pubkeys
        data: data.to_vec(),
    }
}

#[tokio::main]
async fn main() -> Result<()> {

    //=============================================Accounts (3)=================================================
    // 1: first account - transaction signer
    // read keypair, will be used to sign transaction
    let transaction_signer =  read_keypair_file(Path::new(KEYPAIR_PATH)).unwrap();
    println!("Account to create public Key: {}", transaction_signer.pubkey());

    // 2: 2nd - account to create
    let program_id: &Pubkey = &Pubkey::from_str(PROGRAM_ID)?;
    let seed_num = 7_u8;
    let vec = vec![seed_num];

    let seed = "hello_world".as_bytes();

    let (new_pda_key, bump) = Pubkey::find_program_address(&[seed], &program_id);

    println!("new_pda_key: {}", new_pda_key);
    println!("seed: {:?}", seed);
    println!("bump: {}", bump);

    // 3: system_program account
    let sys_prg_pubkey = Pubkey::from_str(SYSTEM_PRG_PUBKEY).unwrap();
    println!("System program account public Key: {}", SYSTEM_PRG_PUBKEY);

    //=============================================Accounts (3)=================================================





    //rpc-client, it will be used to send transaction to solana-validator
    let client = RpcClient::new_with_commitment(
        SOLANA_URL.to_string(),
        CommitmentConfig {
            commitment: CommitmentLevel::Confirmed,
        },
    );

    // data
    let mut data = vec![0];
    data.extend(seed);
    let data_slice = data.as_slice();

    // create instruction
    let ix = build_instruction(&data_slice, transaction_signer.pubkey(), new_pda_key, sys_prg_pubkey);

    // take a look at purpose of the blockhash:
    // https://solana.com/docs/core/transactions#recent-blockhash
    let blockhash = client.get_latest_blockhash().await?;

    // solana tx
    let mut tx =
        Transaction::new_with_payer(&[ix], Some(&transaction_signer.pubkey()));
    tx.sign(&[&transaction_signer], blockhash);

    // let's send it!
    let  sig= client.send_and_confirm_transaction(&tx).await?;

    println!("we have done it, solana signature: {}", sig);

    let config = RpcTransactionConfig {
        //commitment: CommitmentConfig::finalized().into(), // так не работает
        commitment: CommitmentConfig::confirmed().into(),
        encoding: UiTransactionEncoding::Base64.into(),
        max_supported_transaction_version: Some(0),
    };

    let transaction = client.get_transaction_with_config(&sig, config).await?;

    println!("Transaction data is {:#?}", transaction);

    Ok(())
}