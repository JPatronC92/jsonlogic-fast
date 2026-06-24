use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use jsonlogic_fast::b2a::{
    auth::verify_siwe,
    pricing::{calculate_rule_depth, estimate_cost},
};
use jsonlogic_fast::CompiledRule;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

// Simple request ID counter (no extra uuid dep)
static REQ_COUNTER: AtomicU64 = AtomicU64::new(1);

pub mod storage;
pub mod blockchain;
mod hardening;
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
    // Valores esperados (pueden venir de env o config en producci├│n)
    // Para agentes, se recomienda fijar domain y uri estrictos.
    let expected_domain = std::env::var("SIWE_DOMAIN").ok();
    let expected_uri = std::env::var("SIWE_URI").ok();

    // Llamada con verificaci├│n estricta + devuelve el Message completo
    let (wallet_address, siwe_msg) = verify_siwe(
        &payload.message,
        &payload.signature,
        expected_domain.as_deref(),
        expected_uri.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::UNAUTHORIZED, e))?;

    let req_id = format!("req-{}", REQ_COUNTER.fetch_add(1, Ordering::Relaxed));
    tracing::info!(req_id = %req_id, wallet = %wallet_address, "authenticated evaluate request");

    // Ya no re-parseamos: usamos el Message devuelto por verify_siwe
    let nonce_valid = state
        .storage
        .check_and_record_nonce(&wallet_address, siwe_msg.nonce.as_str())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !nonce_valid {
        tracing::warn!(req_id = %req_id, wallet = %wallet_address, "nonce replay detected");
        return Err((StatusCode::UNAUTHORIZED, "Nonce already used (replay attack detected)".into()));
    }

    // Basic rate limiting: 10 requests per minute per wallet.
    // Treat Err (e.g. transient put after allow, or get miss) as allow (fail-open) to avoid 500 for rate.
    // Only explicit false from pure decision -> 429.
    let rate_result = state.storage.check_and_record_request(&wallet_address, 60, 10).await;
    let rate_ok = match rate_result {
        Ok(b) => b,
        Err(_) => true, // fail-open as before (rate errors treated as allow)
    };
    if !rate_ok {
        tracing::warn!(req_id = %req_id, wallet = %wallet_address, "rate limit exceeded");
        return Err((StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".into()));
    }

    tracing::debug!(req_id = %req_id, "rate and nonce passed, computing rule");

    // Compute/evaluate BEFORE charging (high priority audit fix).
    // This guarantees the agent never pays for a failed evaluation.
    // If deduct fails after successful eval, the result is not returned (revenue risk accepted; documented).
    let depth = calculate_rule_depth(&payload.rule);
    let size = if payload.data.is_array() {
        payload.data.as_array().unwrap().len()
    } else {
        1
    };
    let cost = estimate_cost(depth, size);

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

    // Charge only after successful rule evaluation/compilation.
    state.storage.deduct_balance(&wallet_address, cost)
        .await
        .map_err(|e| match e {
            storage::B2AStorageError::InsufficientFunds => (StatusCode::PAYMENT_REQUIRED, "Insufficient funds".to_string()),
            storage::B2AStorageError::Other(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        })?;

    tracing::info!(req_id = %req_id, cost = %cost.to_string(), "deduct successful; returning result");

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
