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
    hash::Hash,
    signature::{Keypair,Signature,Signer},
    pubkey::Pubkey,
    commitment_config::CommitmentConfig,
    message::{Message, v0, VersionedMessage},
    transaction::{Transaction, VersionedTransaction},
    nonce::{State, state},
    system_instruction
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
    let wallet_public_key= wallet.pubkey();
    println!("Public Key of wallet: {}", wallet_public_key.to_string());

    let nonce_key_str = env::var("NONCE_KEY").unwrap();
    let nonce_key_bytes = bs58::decode(nonce_key_str)
        .into_vec().unwrap();
    let nonce_keypair =Arc::new(Keypair::from_bytes(&nonce_key_bytes).unwrap());
    let nonce_public_key= nonce_keypair.pubkey();
    println!("Public Key on NONCE ACCOUNT: {}", nonce_public_key.to_string());


    //Create web3 connection
    let rpc_url = env::var("RPC_API").unwrap();
    let commitment = CommitmentConfig::processed();
    let rpc_client = RpcClient::new_with_commitment(rpc_url.to_string(),commitment);

    ////////////////////////////
    // let nonce_rent = rpc_client.get_minimum_balance_for_rent_exemption(State::size()).unwrap();
    // let instr = system_instruction::create_nonce_account(
    //     &wallet.pubkey(),
    //     &nonce_keypair.pubkey(),
    //     &wallet.pubkey(), // Make the fee wallet the nonce account authority
    //     nonce_rent,
    // );

    // let mut tx = Transaction::new_with_payer(&instr, Some(&wallet.pubkey()));

    // let blockhash = rpc_client.get_latest_blockhash().unwrap();
    // tx.try_sign(&[&nonce_keypair, &wallet], blockhash);

    // let signature=rpc_client.send_and_confirm_transaction(&tx).unwrap();
    // println!("{}", signature);
    ///////////////////////////////////
   



    let nonce_account_data = rpc_client.get_account(&nonce_keypair.pubkey()).unwrap();
    // let (nonce_blockhash, _fee_calculator) = match nonce_account_data {
    //     State::Initialized(data) => (data.blockhash(), data.fee_calculator),
    //     _ => panic!("Nonce account not initialized"),
    // };
    let authority = Pubkey::new_from_array(nonce_account_data.data[8..40].try_into().unwrap());
    let nonce_bytes: [u8; 32] = nonce_account_data.data[40..72].try_into().unwrap();
    // let durable_nonce=bs58::encode(nonce_bytes).into_string();
    let durable_nonce = Hash::new_from_array(nonce_bytes);


    let recent_blockhash = durable_nonce;

    let mut instructions = vec![];


    let nonce_instruction = system_instruction::advance_nonce_account(
        &nonce_keypair.pubkey(),
        &wallet.pubkey(),
    );
    instructions.push(nonce_instruction);

    let transfer_instruction = system_instruction::transfer(
        &wallet.pubkey(),
        &wallet.pubkey(),
        10000
    );
    instructions.push(transfer_instruction);

    let v0_message= v0::Message::try_compile(
        &wallet.pubkey(),
        &instructions,
        &[],
        recent_blockhash,
    ).unwrap();
    let mut v0_transaction=VersionedTransaction::try_new(VersionedMessage::V0(v0_message), &[wallet]).unwrap();

    let result = rpc_client.send_and_confirm_transaction(&v0_transaction).unwrap();


    println!("https://solscan.io/tx/{result}");
}