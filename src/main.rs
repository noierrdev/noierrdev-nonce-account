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

use solana_client::{
    rpc_client::RpcClient,
    tpu_client::{TpuClient, TpuClientConfig},
    rpc_response::RpcContactInfo
};
use solana_client::rpc_config::RpcSendTransactionConfig;
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


#[tokio::main]
async fn main(){
    
    dotenv::dotenv().ok();
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
    let rpc_url = env::var("RPC_API").unwrap_or("http://localhost:8899".to_string());
    let commitment = CommitmentConfig::processed();
    let rpc_client = RpcClient::new_with_commitment(rpc_url.to_string(),commitment);

    let sender_url = env::var("RPC_API").unwrap_or("http://localhost:4040".to_string());
    let sender_client = RpcClient::new_with_commitment(sender_url.to_string(),commitment);

    let config = RpcSendTransactionConfig {
        skip_preflight: true,
        .. RpcSendTransactionConfig::default()
    };
}

pub fn create_nonce_account(rpc_client : &RpcClient, wallet : &Keypair, nonce_keypair :&Keypair)
->VersionedTransaction
{
    let nonce_rent = rpc_client.get_minimum_balance_for_rent_exemption(State::size()).unwrap();
    let create_nonce_instruction = system_instruction::create_nonce_account(
        &wallet.pubkey(),
        &nonce_keypair.pubkey(),
        &wallet.pubkey(),
        nonce_rent,
    );
    let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
    let v0_message= v0::Message::try_compile(
        &wallet.pubkey(),
        &create_nonce_instruction,
        &[],
        recent_blockhash,
    ).unwrap();
    let mut v0_transaction=VersionedTransaction::try_new(VersionedMessage::V0(v0_message), &[wallet, nonce_keypair]).unwrap();
    v0_transaction

}

pub fn test_nonce_account(rpc_client : &RpcClient, wallet : &Keypair, nonce_address : &str)
->VersionedTransaction
{
    let nonce_account = Pubkey::from_str_const(nonce_address);
    let nonce_account_data = rpc_client.get_account(&nonce_account).unwrap();
    let authority = Pubkey::new_from_array(nonce_account_data.data[8..40].try_into().unwrap());
    let nonce_bytes: [u8; 32] = nonce_account_data.data[40..72].try_into().unwrap();
    let durable_nonce = Hash::new_from_array(nonce_bytes);
    let recent_blockhash = durable_nonce;

    let mut instructions = vec![];

    let nonce_instruction = system_instruction::advance_nonce_account(
        &nonce_account,
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
    v0_transaction
}