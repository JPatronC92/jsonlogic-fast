use api::{create_app, AppState};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let state = AppState {
        balances: Arc::new(Mutex::new(std::collections::HashMap::new())),
    };

    let app = create_app(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("API Server listening on 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
