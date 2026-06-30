use alloy_primitives::{Address, U256};
use api::blockchain::{retry_with_backoff, B2AStaking, BlockchainConfig};
use api::storage::{DynamoStorage, MemoryStorage, StorageBackend};
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let balances_table = env::var("BALANCES_TABLE").unwrap_or_else(|_| "B2A_Balances".to_string());
    let nonces_table = env::var("NONCES_TABLE").unwrap_or_else(|_| "B2A_Nonces".to_string());
    let use_dynamo = env::var("USE_DYNAMODB").unwrap_or_else(|_| "false".to_string()) == "true";
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string());

    if environment == "prod" && !use_dynamo {
        panic!("MemoryStorage is not allowed when ENVIRONMENT=prod");
    }

    let storage: Arc<StorageBackend> = if use_dynamo {
        println!("Slasher: Using DynamoDB Storage...");
        Arc::new(StorageBackend::Dynamo(
            DynamoStorage::new(&balances_table, &nonces_table).await,
        ))
    } else {
        println!("Slasher: Using Memory Storage...");
        Arc::new(StorageBackend::Memory(MemoryStorage::new()))
    };

    // Use runtime secret fetch (from_env_with_secret_fallback) so PRIVATE_KEY is not required in env.
    let config = BlockchainConfig::from_env_with_secret_fallback().await;
    let provider = config.write_provider();
    let contract = B2AStaking::new(config.contract_address, provider.clone());

    println!(
        "Starting slasher for contract {}...",
        config.contract_address
    );

    let all_balances = storage.get_all_balances().await?;
    println!(
        "Found {} users off-chain (filtered, no __meta/__rate).",
        all_balances.len()
    );

    for (user_addr_str, offchain_balance) in all_balances {
        // Only user entries (filter done at storage layer via FilterExpression)
        if user_addr_str.starts_with("__") {
            continue;
        }
        if let Ok(address) = user_addr_str.parse::<Address>() {
            // Get on-chain balance - native U256, no conversion
            // Wrapped with retry+backoff for transient RPC failures (Fase 4)
            let onchain_balance: U256 =
                retry_with_backoff(|| async { contract.balances(address).call().await }, 3, 500)
                    .await?;

            println!(
                "User {}: On-chain: {}, Off-chain: {}",
                address, onchain_balance, offchain_balance
            );

            if onchain_balance > offchain_balance {
                let diff = onchain_balance - offchain_balance;

                // Dust threshold: 0.0001 * 10^18 = 100_000_000_000_000 (still using integer)
                let dust_threshold = U256::from(100_000_000_000_000u64);

                if diff > dust_threshold {
                    println!("Slashing {} for user {}", diff, address);
                    // Wrapped send with retry for transient failures (Fase 4 resilience)
                    match retry_with_backoff(
                        || async { contract.slash(address, diff).send().await },
                        3,
                        500,
                    )
                    .await
                    {
                        Ok(tx) => {
                            println!("Slash tx sent: {:?}", tx.tx_hash());
                            // Confirmation watch is best-effort (tx may be already confirmed or chain delay); no full retry to avoid consuming
                            let _ = tx.watch().await;
                            println!("Slash tx confirmed (or timeout on watch)!");
                        }
                        Err(e) => eprintln!("Failed to slash {}: {}", address, e),
                    }
                }
            }
        }
    }

    println!("Slashing round complete.");
    Ok(())
}
