use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use jsonlogic_fast::b2a::{
    auth::verify_siwe,
    pricing::{calculate_rule_depth, estimate_cost},
};
use jsonlogic_fast::CompiledRule;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

pub mod storage;
pub mod blockchain;
use storage::StorageBackend;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<StorageBackend>,
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

pub async fn estimate(Json(payload): Json<EstimateRequest>) -> impl IntoResponse {
    let cost = estimate_cost(payload.rule_depth, payload.batch_size);
    (
        StatusCode::OK,
        Json(EstimateResponse {
            estimated_cost: cost,
        }),
    )
}

pub async fn evaluate(
    State(state): State<AppState>,
    Json(payload): Json<EvaluateRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let wallet_address = verify_siwe(&payload.message, &payload.signature)
        .await
        .map_err(|e| (StatusCode::UNAUTHORIZED, e))?;

    let siwe_msg = siwe::Message::from_str(&payload.message)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid SIWE message: {}", e)))?;

    let nonce_valid = state.storage.check_and_record_nonce(&wallet_address, siwe_msg.nonce.as_str())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if !nonce_valid {
        return Err((StatusCode::UNAUTHORIZED, "Nonce already used (replay attack detected)".into()));
    }

    let depth = calculate_rule_depth(&payload.rule);
    let size = if payload.data.is_array() {
        payload.data.as_array().unwrap().len()
    } else {
        1
    };

    let cost = estimate_cost(depth, size);

    state.storage.deduct_balance(&wallet_address, cost)
        .await
        .map_err(|e| {
            if e == "Insufficient funds" {
                (StatusCode::PAYMENT_REQUIRED, e)
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, e)
            }
        })?;

    let rule_json_str = payload.rule.to_string();
    let data_json_str = payload.data.to_string();

    let rule = CompiledRule::new(&rule_json_str)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid rule: {:?}", e)))?;

    let result = rule.evaluate(&data_json_str).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Evaluation failed: {:?}", e),
        )
    })?;

    let result_value: serde_json::Value =
        serde_json::from_str(&result.to_string()).unwrap_or(serde_json::Value::Null);

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
