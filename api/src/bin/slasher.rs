use api::blockchain::{BlockchainConfig, B2AStaking};
use api::storage::{DynamoStorage, MemoryStorage, StorageBackend};
use alloy::primitives::{Address, U256};
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let balances_table = env::var("BALANCES_TABLE").unwrap_or_else(|_| "B2A_Balances".to_string());
    let nonces_table = env::var("NONCES_TABLE").unwrap_or_else(|_| "B2A_Nonces".to_string());
    let use_dynamo = env::var("USE_DYNAMODB").unwrap_or_else(|_| "false".to_string()) == "true";

    let storage: Arc<StorageBackend> = if use_dynamo {
        println!("Slasher: Using DynamoDB Storage...");
        Arc::new(StorageBackend::Dynamo(DynamoStorage::new(&balances_table, &nonces_table).await))
    } else {
        println!("Slasher: Using Memory Storage...");
        Arc::new(StorageBackend::Memory(MemoryStorage::new()))
    };

    let config = BlockchainConfig::from_env();
    let provider = config.write_provider();
    let contract = B2AStaking::new(config.contract_address, provider.clone());

    println!("Starting slasher for contract {}...", config.contract_address);

    let all_balances = storage.get_all_balances().await?;
    println!("Found {} users off-chain.", all_balances.len());

    for (user_addr_str, offchain_balance) in all_balances {
        if let Ok(address) = user_addr_str.parse::<Address>() {
            // Get on-chain balance
            let onchain_balance_wei = contract.balances(address).call().await?;
            
            // Convert wei to f64
            let onchain_balance_f64 = onchain_balance_wei.to_string().parse::<f64>().unwrap_or(0.0) / 1e18;

            println!("User {}: On-chain: {:.6}, Off-chain: {:.6}", address, onchain_balance_f64, offchain_balance);

            if onchain_balance_f64 > offchain_balance {
                let diff_f64 = onchain_balance_f64 - offchain_balance;
                
                // If the difference is meaningful (avoid floating point dust issues)
                if diff_f64 > 0.0001 {
                    let diff_wei_str = format!("{:.0}", diff_f64 * 1e18);
                    if let Ok(diff_wei) = U256::from_str_radix(&diff_wei_str, 10) {
                        println!("Slashing {} for user {}", diff_wei, address);
                        match contract.slash(address, diff_wei).send().await {
                            Ok(tx) => {
                                println!("Slash tx sent: {:?}", tx.tx_hash());
                                let _ = tx.watch().await;
                                println!("Slash tx confirmed!");
                            }
                            Err(e) => eprintln!("Failed to slash {}: {}", address, e),
                        }
                    }
                }
            }
        }
    }

    println!("Slashing round complete.");
    Ok(())
}
