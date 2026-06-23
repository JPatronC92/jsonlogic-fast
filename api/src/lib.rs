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
    /// Cost returned as decimal string to preserve full U256 precision (no float loss)
    pub cost: String,
}

#[derive(Deserialize)]
pub struct EstimateRequest {
    pub rule_depth: usize,
    pub batch_size: usize,
}

#[derive(Serialize)]
pub struct EstimateResponse {
    /// Cost returned as decimal string (18 decimals unit)
    pub estimated_cost: String,
}

pub async fn estimate(Json(payload): Json<EstimateRequest>) -> impl IntoResponse {
    let cost = estimate_cost(payload.rule_depth, payload.batch_size);
    (
        StatusCode::OK,
        Json(EstimateResponse {
            estimated_cost: cost.to_string(),
        }),
    )
}

pub async fn evaluate(
    State(state): State<AppState>,
    Json(payload): Json<EvaluateRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Valores esperados (pueden venir de env o config en producción)
    // Para agentes, se recomienda fijar domain y uri estrictos.
    let expected_domain = std::env::var("SIWE_DOMAIN").ok();
    let expected_uri = std::env::var("SIWE_URI").ok();

    // Llamada con verificación estricta + devuelve el Message completo
    let (wallet_address, siwe_msg) = verify_siwe(
        &payload.message,
        &payload.signature,
        expected_domain.as_deref(),
        expected_uri.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::UNAUTHORIZED, e))?;

    // Ya no re-parseamos: usamos el Message devuelto por verify_siwe
    let nonce_valid = state
        .storage
        .check_and_record_nonce(&wallet_address, siwe_msg.nonce.as_str())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if !nonce_valid {
        return Err((StatusCode::UNAUTHORIZED, "Nonce already used (replay attack detected)".into()));
    }

    // Basic rate limiting: 10 requests per minute per wallet
    let rate_ok = state.storage.check_and_record_request(&wallet_address, 60, 10)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if !rate_ok {
        return Err((StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".into()));
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
            cost: cost.to_string(),
        }),
    ))
}

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/v1/estimate", post(estimate))
        .route("/v1/evaluate", post(evaluate))
        .with_state(state)
}
