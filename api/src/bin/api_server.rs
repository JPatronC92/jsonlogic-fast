use api::storage::{DynamoStorage, MemoryStorage, StorageBackend};
use api::{create_app, AppState};
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber so info!/warn! in evaluate handler produce output (matches lambda.rs)
    tracing_subscriber::fmt::init();
    let balances_table = env::var("BALANCES_TABLE").unwrap_or_else(|_| "B2A_Balances".to_string());
    let nonces_table = env::var("NONCES_TABLE").unwrap_or_else(|_| "B2A_Nonces".to_string());

    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string());

    let use_dynamo = env::var("USE_DYNAMODB").unwrap_or_else(|_| "false".to_string()) == "true";

    let state = if use_dynamo {
        println!("Using DynamoDB Storage...");
        let storage = DynamoStorage::new(&balances_table, &nonces_table).await;
        AppState {
            storage: Arc::new(StorageBackend::Dynamo(storage)),
        }
    } else {
        if environment == "prod" {
            panic!("MemoryStorage is not allowed when ENVIRONMENT=prod");
        }
        println!("Using Memory Storage...");
        AppState {
            storage: Arc::new(StorageBackend::Memory(MemoryStorage::new())),
        }
    };

    let app = create_app(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("API Server listening on 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
