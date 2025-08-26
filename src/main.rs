mod program_option;

use {
    crate::program_option::{Args, TransactionType},
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
//const KEYPAIR_PATH: &str = "/home/kvasshtain/.config/solana/id.json";

// solana is running on local machine
const SOLANA_URL: &str = "http://localhost:8899";
// program_id of the "memo" program
// https://github.com/solana-program/memo/blob/37568de8dae6a4e69572a85e8c166910da232b90/program/src/lib.rs#L26
const PROGRAM_ID: &str = "4fnvoc7wADwtwJ9SRUvL7KpCBTp8qztm5GqjZBFP7GTt";

pub fn build_ix(data: &[u8], payer: Pubkey, pubkeys: &[&Pubkey]) -> Instruction {

    let mut accounts: Vec<AccountMeta> = pubkeys
        .iter()
        .map(|&pubkey| AccountMeta::new(*pubkey, false))
        .collect();

    accounts.insert(0, AccountMeta::new(payer, true));
    accounts.push(AccountMeta::new_readonly(solana_sdk::system_program::id(), false));

    Instruction {
        program_id: Pubkey::from_str(PROGRAM_ID).unwrap(),
        accounts,
        data: data.to_vec(),
    }
}

pub async fn build_tx(
    data: Vec<u8>,
    client: &RpcClient,
    payer: Keypair,
    account: Pubkey,
) -> Result<Signature> {
    let payer_key = payer.pubkey();

    let ix = build_ix(&data.as_slice(), payer_key, &[&account]);

    let blockhash = client.get_latest_blockhash().await?;

    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer_key));

    tx.sign(&[&payer], blockhash);

    Ok(client.send_and_confirm_transaction(&tx).await?)
}

pub async fn build_transfer_from_tx(
    data: Vec<u8>,
    client: &RpcClient,
    payer: Keypair,
    from: Pubkey,
    to: Pubkey,
) -> Result<Signature> {
    let payer_key = payer.pubkey();

    let ix = build_ix(&data.as_slice(), payer_key, &[&from, &to]);

    let blockhash = client.get_latest_blockhash().await?;

    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer_key));

    tx.sign(&[&payer], blockhash);

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
    let tx_sig = read_keypair_file(Path::new(args.keypair_path.as_str())).unwrap();

    // 2: 2nd - account to create
    let program_id: &Pubkey = &Pubkey::from_str(PROGRAM_ID)?;

    let mut data = vec![args.mode.clone() as u8];

    let sig = match args.mode {
        TransactionType::Create => {
            let seed = args.seed.unwrap();
            data.extend(seed.as_bytes());
            let (new, _bump) = Pubkey::find_program_address(&[&*seed.as_bytes()], &program_id);
            build_tx(data, &client, tx_sig, new).await?
        }
        TransactionType::Resize => {
            data.extend(args.size.unwrap().to_le_bytes());
            let (resized, _bump) = Pubkey::find_program_address(&[&*args.seed.unwrap().as_bytes()], &program_id);
            build_tx(data, &client, tx_sig, resized).await?
        }
        TransactionType::Transfer => {
            data.extend(args.amount.unwrap().to_le_bytes());
            build_tx(data, &client, tx_sig, Pubkey::from_str(args.to.unwrap().as_str())?).await?
        }
        TransactionType::TransferFrom => {
            data.extend(args.amount.unwrap().to_le_bytes());
            let seed = args.seed.unwrap();
            data.extend(seed.as_bytes());
            let (from, _bump) = Pubkey::find_program_address(&[&*seed.as_bytes()], &program_id);
            build_transfer_from_tx(data, &client, tx_sig, from, Pubkey::from_str(args.to.unwrap().as_str())?).await?
        }
        TransactionType::Allocate => {
            data.extend(args.size.unwrap().to_le_bytes());
            let seed = args.seed.unwrap();
            data.extend(seed.as_bytes());
            let (resized, _bump) = Pubkey::find_program_address(&[&*seed.as_bytes()], &program_id);
            build_tx(data, &client, tx_sig, resized).await?
        }
    };

    println!("job has been done, solana signature: {}", sig);

    Ok(sig)
}

pub async fn show_tx_data(client: &RpcClient, sig: Signature) -> Result<()> {
    let tx_data = read_transaction(client, sig).await?;

    println!("Transaction data is {:#?}", tx_data);

    Ok(())
}

pub async fn execute(args: Args) -> Result<()> {
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
