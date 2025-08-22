mod program_option;

use {
    crate::program_option::{Args, ModeType},
    anyhow::Result,
    clap::Parser,
    solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcTransactionConfig},
    solana_sdk::{
        commitment_config::{CommitmentConfig, CommitmentLevel},
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::{Keypair, Signature, Signer, read_keypair_file},
        transaction::Transaction,
    },
    solana_transaction_status_client_types::{
        EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding,
    },
    std::{path::Path, str::FromStr},
};

// path to keypair
const KEYPAIR_PATH: &str = "/home/kvasshtain/.config/solana/id.json";
// solana is running on local machine
const SOLANA_URL: &str = "http://localhost:8899";
// program_id of the "memo" program
// https://github.com/solana-program/memo/blob/37568de8dae6a4e69572a85e8c166910da232b90/program/src/lib.rs#L26
const PROGRAM_ID: &str = "4fnvoc7wADwtwJ9SRUvL7KpCBTp8qztm5GqjZBFP7GTt";

// create instuction of the memo-program
pub fn build_ix(data: &[u8], payer: Pubkey, new: Pubkey) -> Instruction {
    Instruction {
        program_id: Pubkey::from_str(PROGRAM_ID).unwrap(),
        accounts: Vec::from([
            AccountMeta::new(payer, true),
            AccountMeta::new(new, false),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ]), //accounts_pubkeys
        data: data.to_vec(),
    }
}

pub async fn send_ix(
    data: Vec<u8>,
    client: &RpcClient,
    payer: Keypair,
    new: Pubkey,
) -> Result<Signature> {
    let payer_key = payer.pubkey();
    // create instruction
    let ix = build_ix(&data.as_slice(), payer_key, new);

    // take a look at purpose of the blockhash:
    // https://solana.com/docs/core/transactions#recent-blockhash
    let blockhash = client.get_latest_blockhash().await?;

    // solana tx
    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer_key));

    tx.sign(&[&payer], blockhash);

    // let's send it!
    Ok(client.send_and_confirm_transaction(&tx).await?)
}

pub async fn read_transaction(
    client: &RpcClient,
    sig: Signature,
) -> Result<EncodedConfirmedTransactionWithStatusMeta> {
    let config = RpcTransactionConfig {
        commitment: CommitmentConfig::confirmed().into(),
        encoding: UiTransactionEncoding::Base64.into(),
        max_supported_transaction_version: Some(0),
    };

    Ok(client.get_transaction_with_config(&sig, config).await?)
}

pub async fn send_tx(args: Args, client: &RpcClient) -> Result<Signature> {
    let tx_sig = read_keypair_file(Path::new(KEYPAIR_PATH)).unwrap();

    // 2: 2nd - account to create
    let program_id: &Pubkey = &Pubkey::from_str(PROGRAM_ID)?;

    let seed = args.seed.as_bytes();
    let (new_pda_key, _bump) = Pubkey::find_program_address(&[&*seed], &program_id);

    //let mode = args.mode;
    let mut data: Vec<u8> = vec![args.mode.clone() as u8];

    match args.mode {
        ModeType::Create => {
            data.extend(seed);
        }
        ModeType::Resize => {
            data.extend(args.size.unwrap().to_le_bytes());
        }
        ModeType::Send => {
            data.extend(args.amount.unwrap().to_le_bytes());
        }
    }

    // let's send it!
    let sig = send_ix(data, &client, tx_sig, new_pda_key).await?;

    println!("we have done it, solana signature: {}", sig);

    Ok(sig)
}

pub async fn show_tx_data(client: &RpcClient, sig: Signature) -> Result<()> {
    let tx_data = read_transaction(client, sig).await?;

    println!("Transaction data is {:#?}", tx_data);

    Ok(())
}

pub async fn execute(args: Args) -> Result<()> {
    //rpc-client, it will be used to send transaction to solana-validator
    let client = RpcClient::new_with_commitment(
        SOLANA_URL.to_string(),
        CommitmentConfig {
            commitment: CommitmentLevel::Confirmed,
        },
    );

    let sig = send_tx(args, &client).await?;

    println!("we have done it, solana signature: {}", sig);

    show_tx_data(&client, sig).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    execute(Args::parse()).await?;
    Ok(())
}
