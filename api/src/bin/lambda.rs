use api::{create_app, AppState};
use std::sync::Arc;
use tokio::sync::Mutex;
use lambda_http::{run, tracing, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    let state = AppState {
        balances: Arc::new(Mutex::new(std::collections::HashMap::new())),
    };

    let app = create_app(state);

    run(app).await
}
