pub mod config;

use std::{
    path::PathBuf,
    sync::Arc,
    fs::OpenOptions,
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
    io::Read,
};
use clap::Parser;

use alloy_network::Network;
use alloy_network::eip2718::Encodable2718;
use alloy_primitives::Address;
use alloy_provider::{Provider, ProviderBuilder, WsConnect};
use alloy_signer_local::PrivateKeySigner;
use alloy_consensus::TxEnvelope;
use config::CTFConfig;
use eyre::eyre::{Result, eyre};
use pbh_ctf::{
    CTFTransactionBuilder, PBH_CTF_CONTRACT, PBH_ENTRY_POINT,
    bindings::IPBHEntryPoint::IPBHEntryPointInstance,
    world_id::WorldID,
};
use reqwest::Url;

// Add CLI args struct
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = 350_000)]
    gas_limit: u64,
    
    #[arg(long, default_value_t = 10)]
    iterations_pbh: u64,
    
    #[arg(long, default_value_t = 10)]
    iterations: u64,

    #[arg(long, default_value_t = 4)]
    n: u64,

    #[arg(short, long, default_value_t = false)]
    reverse: bool,
}

// Function to log events to pbh.log
fn now() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
    timestamp.to_string()
}

fn log_event(event: &str) -> Result<()> {
    let log_entry = format!("{} {}\n", now(), event);
    
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("pbh.log")?;
    
    file.write_all(log_entry.as_bytes())?;
    Ok(())
}

// Function to read PBH nonce from file
fn read_pbh_nonce() -> Result<u16> {
    let mut file = match OpenOptions::new()
        .read(true)
        .open("./pbh.nonce") {
            Ok(file) => file,
            Err(_) => return Ok(0), // Return 0 if file doesn't exist
        };
    
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    Ok(contents.trim().parse::<u16>()?)
}

// Function to write PBH nonce to file
fn write_pbh_nonce(nonce: u16) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("./pbh.nonce")?;
    
    write!(file, "{}", nonce)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let args = Args::parse();

    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bin/pbh_ctf.toml");
    let config = CTFConfig::load(Some(config_path.as_path()))?;
    let private_key_0 = std::env::var("PRIVATE_KEY_0")?;
    let private_key_1 = std::env::var("PRIVATE_KEY_1")?;
    let signer_0 = private_key_0.parse::<PrivateKeySigner>()?;
    let signer_1 = private_key_1.parse::<PrivateKeySigner>()?;
    let provider = Arc::new(
        ProviderBuilder::new()
        .on_http(config.provider_uri.parse::<Url>()?)
    );
    tracing::info!("signer_0.address(): {}", signer_0.address());
    tracing::info!("signer_1.address(): {}", signer_1.address());

    let world_id = WorldID::new(&config.semaphore_secret)?;

    let pbh_entrypoint = IPBHEntryPointInstance::new(PBH_ENTRY_POINT, provider.clone());
    let pbh_nonce_limit = pbh_entrypoint.numPbhPerMonth().call().await?._0;
    tracing::info!("pbh_nonce_limit: {}", pbh_nonce_limit);

    let player = config.player_address;
    let mut pbh_nonce = read_pbh_nonce()?; // Read initial nonce from file
    tracing::info!("pbh_nonce: {}", pbh_nonce);
    
    let mut wallet_nonce_0 = provider.get_transaction_count(signer_0.address()).await?;
    let mut wallet_nonce_1 = provider.get_transaction_count(signer_1.address()).await?;
    tracing::info!("wallet_nonce_0: {}", wallet_nonce_0);
    tracing::info!("wallet_nonce_1: {}", wallet_nonce_1);

    async fn prepare_pbh_ctf_transaction(
        player: Address,
        wallet_nonce: u64,
        signer: PrivateKeySigner,
        world_id: &WorldID,
        pbh_nonce: u16,
        gas_limit: u64,
        iterations: u64,
    ) -> Result<TxEnvelope> {
        tracing::info!("Preparing PBH CTF transaction");
        let calls = pbh_ctf::client_contract_multicall(player, iterations, PBH_CTF_CONTRACT);
        let tx = CTFTransactionBuilder::new()
            .gas_limit(gas_limit)
            .to(PBH_ENTRY_POINT)
            .nonce(wallet_nonce)
            .from(signer.address())
            .max_fee_per_gas(2e8 as u128)
            .max_priority_fee_per_gas(1e8 as u128)
            .with_pbh_multicall(world_id, pbh_nonce, signer.address(), calls)
            .await?
            .build(signer.clone())
            .await?;
        
        Ok(tx)
    }
    async fn prepare_ctf_transaction(
        player: Address,
        wallet_nonce: u64,
        signer: PrivateKeySigner,
        gas_limit: u64,
        iterations: u64,
    ) -> Result<TxEnvelope> {
        tracing::info!("Preparing CTF transaction");
        let calldata = pbh_ctf::client_contract_calldata(player, iterations);
        let tx = CTFTransactionBuilder::new()
            .gas_limit(gas_limit)
            .to(PBH_CTF_CONTRACT)
            .nonce(wallet_nonce)
            .from(signer.address())
            .max_fee_per_gas(15e8 as u128)
            .max_priority_fee_per_gas(10e8 as u128)
            .input(calldata.into())
            .build(signer.clone())
            .await?;
        Ok(tx)
    }

    let mut txs: Vec<TxEnvelope> = Vec::new();
    for i in 0..args.n {
        let is_0 = i % 2 == 0;
        let is_pbh = i % 4 > 1;
        let signer = if is_0 { signer_0.clone() } else { signer_1.clone() };
        let wallet_nonce = if is_0 { wallet_nonce_0 } else { wallet_nonce_1 };
        let tx = if is_pbh {
            prepare_pbh_ctf_transaction(player, wallet_nonce, signer, &world_id, pbh_nonce, args.gas_limit, args.iterations_pbh).await?
        } else {
            prepare_ctf_transaction(player, wallet_nonce, signer, args.gas_limit, args.iterations).await?
        };
        txs.push(tx);
        if is_pbh {
            pbh_nonce += 1;
            write_pbh_nonce(pbh_nonce)?;
        };
        if is_0 {
            wallet_nonce_0 += 1;
        } else {
            wallet_nonce_1 += 1;
        }
    }

    let mut txs = txs;
    if args.reverse && txs.len() > 1 {
        let first_two = txs.drain(0..2).collect::<Vec<_>>();
        txs.extend(first_two);
    }
    
    for tx in txs.iter() {
        tracing::info!("Sending transaction: {:?}", tx);
        let tx_encoded = tx.encoded_2718();
        let provider_clone = provider.clone();
        tokio::spawn(async move {
            match provider_clone.send_raw_transaction(&tx_encoded).await {
                Ok(pending_tx) => {
                    tracing::info!("Sent transaction: {:?}", pending_tx.tx_hash());
                    if let Err(e) = log_event(&format!("TxSubmitted {} {:?}", 999, pending_tx.tx_hash())) {
                        tracing::error!("Failed to log transaction event: {:?}", e);
                    }
                },
                Err(e) => {
                    tracing::error!("Failed to send transaction: {:?}", e);
                    if let Err(log_err) = log_event(&format!("TxFailed {} {:?}", 999, e)) {
                        tracing::error!("Failed to log error event: {:?}", log_err);
                    }
                }
            }
        });
        // Wait to work around Alchemy rate limiting
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    };
    tokio::time::sleep(std::time::Duration::from_millis(6000)).await;

    Ok(())
}

async fn _get_pbh_nonce<P: Provider<N>, N: Network>(
    world_id: &WorldID,
    provider: P,
    max_pbh_nonce: u16,
) -> Result<u16> {
    let start_nonce = 0;
    let pbh_entrypoint_instance = IPBHEntryPointInstance::new(PBH_ENTRY_POINT, provider);
    for i in start_nonce..=max_pbh_nonce {
        let nullifier_hash = world_id.pbh_ext_nullifier(i).2;
        let is_used = pbh_entrypoint_instance
            .nullifierHashes(nullifier_hash)
            .call()
            .await?
            ._0;
        if !is_used {
            return Ok(i);
        }
    }

    Err(eyre!("No available PBH nonce"))
}
