use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use jsonlogic_fast::b2a::{auth::verify_siwe, pricing::estimate_cost};
use jsonlogic_fast::CompiledRule;

#[derive(Clone)]
pub struct AppState {
    pub balances: Arc<Mutex<std::collections::HashMap<String, f64>>>,
}

#[derive(Deserialize)]
pub struct EvaluateRequest {
    pub message: String,
    pub signature: String,
    pub rule: serde_json::Value,
    pub data: serde_json::Value,
}

#[derive(Serialize)]
pub struct EvaluateResponse {
    pub result: serde_json::Value,
    pub cost: f64,
}

#[derive(Deserialize)]
pub struct EstimateRequest {
    pub rule_depth: usize,
    pub batch_size: usize,
}

#[derive(Serialize)]
pub struct EstimateResponse {
    pub estimated_cost: f64,
}

pub async fn estimate(
    Json(payload): Json<EstimateRequest>,
) -> impl IntoResponse {
    let cost = estimate_cost(payload.rule_depth, payload.batch_size);
    (StatusCode::OK, Json(EstimateResponse { estimated_cost: cost }))
}

pub async fn evaluate(
    State(state): State<AppState>,
    Json(payload): Json<EvaluateRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let wallet_address = verify_siwe(&payload.message, &payload.signature).await
        .map_err(|e| (StatusCode::UNAUTHORIZED, e))?;

    let depth = 1;
    let size = if payload.data.is_array() {
        payload.data.as_array().unwrap().len()
    } else {
        1
    };

    let cost = estimate_cost(depth, size);

    {
        let mut balances = state.balances.lock().await;
        let balance = balances.entry(wallet_address.clone()).or_insert(10.0);
        if *balance < cost {
            return Err((StatusCode::PAYMENT_REQUIRED, "Insufficient funds".into()));
        }
        *balance -= cost;
    }

    let rule_json_str = payload.rule.to_string();
    let data_json_str = payload.data.to_string();

    let rule = CompiledRule::new(&rule_json_str)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid rule: {:?}", e)))?;

    let result = rule.evaluate(&data_json_str)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Evaluation failed: {:?}", e)))?;

    let result_value: serde_json::Value = serde_json::from_str(&result.to_string())
        .unwrap_or(serde_json::Value::Null);

    Ok((
        StatusCode::OK,
        Json(EvaluateResponse {
            result: result_value,
            cost,
        }),
    ))
}

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/v1/estimate", post(estimate))
        .route("/v1/evaluate", post(evaluate))
        .with_state(state)
}
