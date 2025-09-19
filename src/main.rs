use anyhow::{Context, Result};
use clap::Parser;
use futures::{ stream::StreamExt};

use std::{
    env, collections::{HashMap, HashSet},
    time::{ Instant, SystemTime, UNIX_EPOCH},
    // fs::{self, File}, path::Path,
    fs::{OpenOptions},
    fs,
    sync::Arc,
    io::{BufRead,Write,stdin},
    net::{SocketAddr, IpAddr, Ipv4Addr},
    os::unix::net::UnixStream,
    str::FromStr
};
use tokio::{
    time::{sleep, Duration},
    sync::Mutex,
    sync::RwLock
};


use bincode;
use hex;

use reqwest::Client;

use serde_json::json;
use serde_json::Value;
use serde::{Serialize, Deserialize};

use solana_client::{
    rpc_client::RpcClient,
    tpu_client::{TpuClient, TpuClientConfig},
    rpc_response::RpcContactInfo
};
use solana_sdk::{
    bs58,
    signature::{Keypair,Signature,Signer},
    pubkey::Pubkey,
    commitment_config::CommitmentConfig,
    transaction::{VersionedTransaction, Transaction},
    nonce::State
};
use solana_transaction_status::UiTransactionEncoding;

use spl_associated_token_account::{get_associated_token_address, get_associated_token_address_with_program_id};
use spl_token_2022::{id as token_2022_program_id};

use rustls::{
    // ClientConfig as RustlsConfig,
    // crypto::CryptoProvider,
    crypto::ring::default_provider as crypto_default_provider,
    pki_types::PrivatePkcs8KeyDer,
    pki_types::CertificateDer,
    RootCertStore
};

#[tokio::main]
async fn main(){

    let crypto_provider= crypto_default_provider();
    crypto_provider.install_default();
    
    dotenv::dotenv().ok();
    let http_client=Client::new();

    let http_client_escape=http_client.clone();

    //Initialize wallet from private key of .env
    let private_key_str = env::var("PRIVATE_KEY").unwrap();
    let private_key_bytes = bs58::decode(private_key_str)
        .into_vec().unwrap();
    let wallet =Arc::new(Keypair::from_bytes(&private_key_bytes).unwrap());
    let public_key= wallet.pubkey();
    println!("Public Key: {}", public_key.to_string());

    let wallet_monitor=wallet.clone();
    let wallet_escape=wallet.clone();


    //Create web3 connection
    let rpc_url = env::var("RPC_API").unwrap();
    let commitment = CommitmentConfig::processed();
    let rpc_client = RpcClient::new_with_commitment(rpc_url.to_string(),commitment);

    let nonce_account = Keypair::new();

    let nonce_rent = rpc_client.get_minimum_balance_for_rent_exemption(State::size()).unwrap();
    let instr = system_instruction::create_nonce_account(
        &wallet.pubkey(),
        &nonce_account.pubkey(),
        &wallet.pubkey(), // Make the fee wallet the nonce account authority
        nonce_rent,
    );

    let mut tx = Transaction::new_with_payer(&instr, Some(&wallet.pubkey()));

    let blockhash = rpc_client.get_latest_blockhash().unwrap();
    tx.try_sign(&[&nonce_account, wallet], blockhash);


}