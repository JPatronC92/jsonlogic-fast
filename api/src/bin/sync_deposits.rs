use api::blockchain::{BlockchainConfig, B2AStaking};
use api::storage::{DynamoStorage, MemoryStorage, StorageBackend};
use alloy::providers::Provider;
use alloy::rpc::types::eth::Filter;
use alloy::sol_types::SolEvent;
use std::env;
use std::sync::Arc;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let balances_table = env::var("BALANCES_TABLE").unwrap_or_else(|_| "B2A_Balances".to_string());
    let nonces_table = env::var("NONCES_TABLE").unwrap_or_else(|_| "B2A_Nonces".to_string());
    let use_dynamo = env::var("USE_DYNAMODB").unwrap_or_else(|_| "false".to_string()) == "true";

    let storage: Arc<StorageBackend> = if use_dynamo {
        println!("Sync Service: Using DynamoDB Storage...");
        Arc::new(StorageBackend::Dynamo(DynamoStorage::new(&balances_table, &nonces_table).await))
    } else {
        println!("Sync Service: Using Memory Storage...");
        Arc::new(StorageBackend::Memory(MemoryStorage::new()))
    };

    let config = BlockchainConfig::from_env();
    let provider = config.read_provider();

    println!("Starting deposit sync service for contract {}...", config.contract_address);

    let filter = Filter::new()
        .address(config.contract_address)
        .event_signature(B2AStaking::Deposited::SIGNATURE_HASH);

    // Watch for new logs
    let sub = provider.subscribe_logs(&filter).await?;
    let mut stream = sub.into_stream();

    while let Some(log) = stream.next().await {
        if let Ok(decoded) = log.log_decode::<B2AStaking::Deposited>() {
            let event = decoded.inner.data;
            let user = event.user.to_string();
            // Scale by 1e18 if assuming 18 decimals
            let raw_amount = event.amount.to_string().parse::<f64>().unwrap_or(0.0);
            let amount = raw_amount / 1e18; 
            
            println!("Received deposit: {} from {}", amount, user);

            if let Err(e) = storage.add_balance(&user, amount).await {
                eprintln!("Failed to update balance for {}: {}", user, e);
            }
        }
    }

    Ok(())
}
