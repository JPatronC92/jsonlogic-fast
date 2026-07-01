use api::api_key_auth::ApiKeyRecord;
use api::storage::{DynamoStorage, MemoryStorage, StorageBackend};
use rand::{distributions::Alphanumeric, Rng};
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    let command = args[1].as_str();

    let mut environment = None;
    let mut owner = None;
    let mut plan = None;
    let mut hash = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--environment" => {
                if i + 1 < args.len() {
                    environment = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--owner" => {
                if i + 1 < args.len() {
                    owner = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--plan" => {
                if i + 1 < args.len() {
                    plan = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--hash" => {
                if i + 1 < args.len() {
                    hash = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let environment = match environment {
        Some(e) => e,
        None => {
            eprintln!("Error: --environment must be specified (dev, staging, prod)");
            return;
        }
    };

    let balances_table = env::var("BALANCES_TABLE").unwrap_or_else(|_| "B2A_Balances".to_string());
    let nonces_table = env::var("NONCES_TABLE").unwrap_or_else(|_| "B2A_Nonces".to_string());
    let use_dynamo = env::var("USE_DYNAMODB").unwrap_or_else(|_| "true".to_string()) == "true";

    let storage: Arc<StorageBackend> = if use_dynamo {
        println!("Using DynamoDB Storage...");
        Arc::new(StorageBackend::Dynamo(
            DynamoStorage::new(&balances_table, &nonces_table).await,
        ))
    } else {
        if environment == "prod" {
            panic!("MemoryStorage is not allowed when ENVIRONMENT=prod");
        }
        println!("Using Memory Storage...");
        Arc::new(StorageBackend::Memory(MemoryStorage::new()))
    };

    match command {
        "create" => {
            let owner = match owner {
                Some(o) => o,
                None => {
                    eprintln!("Error: --owner must be specified for create");
                    return;
                }
            };
            let plan = match plan {
                Some(p) => p,
                None => {
                    eprintln!("Error: --plan must be specified for create");
                    return;
                }
            };

            let prefix = if environment == "prod" {
                "b2a_live_"
            } else {
                "b2a_beta_"
            };
            let random_str: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(32)
                .map(char::from)
                .collect();
            let raw_key = format!("{}{}", prefix, random_str);

            let mut record = ApiKeyRecord::free_tier(&raw_key, &owner);
            record.plan = plan;

            if let Err(e) = storage.put_api_key(record).await {
                eprintln!("Failed to store API key: {:?}", e);
                return;
            }

            println!("===================================================");
            println!("API Key created successfully.");
            println!("Raw Key (SHOWING ONCE, SAVE IT): {}", raw_key);
            println!("API Key Hash: {}", api::api_key_auth::hash_api_key(&raw_key));
            println!("Owner: {}", owner);
            println!("Environment: {}", environment);
            println!("===================================================");
        }
        "revoke" => {
            let hash = match hash {
                Some(h) => h,
                None => {
                    eprintln!("Error: --hash must be specified for revoke");
                    return;
                }
            };
            match storage.get_api_key(&hash).await {
                Ok(Some(mut record)) => {
                    record.active = false;
                    if let Err(e) = storage.put_api_key(record).await {
                        eprintln!("Failed to update API key status: {:?}", e);
                    } else {
                        println!("API Key {} revoked successfully.", hash);
                    }
                }
                Ok(None) => {
                    println!("API Key hash not found.");
                }
                Err(e) => {
                    eprintln!("Failed to fetch API key: {:?}", e);
                }
            }
        }
        "inspect" => {
            let hash = match hash {
                Some(h) => h,
                None => {
                    eprintln!("Error: --hash must be specified for inspect");
                    return;
                }
            };
            match storage.get_api_key(&hash).await {
                Ok(Some(record)) => {
                    println!("API Key Record:");
                    println!("  Hash: {}", record.api_key_hash);
                    println!("  Owner: {}", record.owner);
                    println!("  Plan: {}", record.plan);
                    println!("  Active: {}", record.active);
                    println!("  Monthly Limit: {}", record.monthly_limit);
                    println!("  Used This Month: {}", record.used_this_month);
                    println!("  Rate Limit Per Minute: {}", record.rate_limit_per_minute);
                }
                Ok(None) => {
                    println!("API Key hash not found.");
                }
                Err(e) => {
                    eprintln!("Failed to fetch API key: {:?}", e);
                }
            }
        }
        _ => {
            println!("Unknown command: {}", command);
            print_usage();
        }
    }
}

fn print_usage() {
    println!("Usage:");
    println!("  cargo run -p api --bin api_key_admin -- create --environment <dev|staging|prod> --owner <owner> --plan <plan>");
    println!("  cargo run -p api --bin api_key_admin -- revoke --environment <dev|staging|prod> --hash <hash>");
    println!("  cargo run -p api --bin api_key_admin -- inspect --environment <dev|staging|prod> --hash <hash>");
}
