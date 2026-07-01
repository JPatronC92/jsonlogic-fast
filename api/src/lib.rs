use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use jsonlogic_fast::b2a::{
    auth::verify_siwe,
    pricing::{calculate_rule_depth, estimate_cost},
};
use jsonlogic_fast::CompiledRule;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

// Simple request ID counter (no extra uuid dep)
static REQ_COUNTER: AtomicU64 = AtomicU64::new(1);

pub mod api_key_auth;
pub mod blockchain;
mod hardening;
pub mod storage;
use api_key_auth::ApiKeyAuth;
use storage::StorageBackend;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<StorageBackend>,
}

#[derive(Deserialize)]
pub struct EvaluateRequest {
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub signature: Option<String>,
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

#[derive(Serialize)]
pub struct UsageResponse {
    pub owner: String,
    pub plan: String,
    pub monthly_limit: u64,
    pub used_this_month: u64,
    pub remaining_this_month: u64,
    pub rate_limit_per_minute: u32,
}

pub async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({"status":"ok","service":"jsonlogic-fast-b2a"})),
    )
}

pub async fn usage(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let req_id = format!("req-{}", REQ_COUNTER.fetch_add(1, Ordering::Relaxed));
    let key = ApiKeyAuth::from_headers(&headers).ok_or((
        StatusCode::UNAUTHORIZED,
        json_error("missing_bearer_token", "Missing bearer token", &req_id),
    ))?;
    let record = state
        .storage
        .get_api_key(&key.hash)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("internal_error", &e.to_string(), &req_id),
            )
        })?
        .filter(|r| r.active)
        .ok_or((
            StatusCode::UNAUTHORIZED,
            json_error("invalid_api_key", "Invalid API key", &req_id),
        ))?;
    Ok((
        StatusCode::OK,
        Json(UsageResponse {
            owner: record.owner,
            plan: record.plan,
            monthly_limit: record.monthly_limit,
            used_this_month: record.used_this_month,
            remaining_this_month: record.monthly_limit.saturating_sub(record.used_this_month),
            rate_limit_per_minute: record.rate_limit_per_minute,
        }),
    ))
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

fn json_error(code: &str, message: &str, req_id: &str) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "error": {
            "code": code,
            "message": message,
            "request_id": req_id
        }
    }))
}

pub async fn evaluate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<EvaluateRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let req_id = format!("req-{}", REQ_COUNTER.fetch_add(1, Ordering::Relaxed));
    let depth = calculate_rule_depth(&payload.rule);
    let contexts: Vec<String> = match payload.data.as_array() {
        Some(items) => items.iter().map(serde_json::Value::to_string).collect(),
        None => vec![payload.data.to_string()],
    };
    let evaluations = contexts.len() as u64;
    let cost = estimate_cost(depth, contexts.len());

    let rule = CompiledRule::new(&payload.rule.to_string()).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            json_error("invalid_rule", &format!("Invalid rule: {:?}", e), &req_id),
        )
    })?;

    let api_key_auth = ApiKeyAuth::from_headers(&headers);
    let wallet_address = if let Some(api_key) = api_key_auth {
        let record = state
            .storage
            .get_api_key(&api_key.hash)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json_error("internal_error", &e.to_string(), &req_id),
                )
            })?
            .filter(|r| r.active)
            .ok_or((
                StatusCode::UNAUTHORIZED,
                json_error("invalid_api_key", "Invalid API key", &req_id),
            ))?;

        let rate_ok = state
            .storage
            .check_and_record_request(
                &format!("api_key:{}", api_key.hash),
                60,
                record.rate_limit_per_minute,
            )
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json_error("internal_error", &e.to_string(), &req_id),
                )
            })?;
        if !rate_ok {
            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                json_error("rate_limit_exceeded", "Rate limit exceeded", &req_id),
            ));
        }
        state
            .storage
            .increment_api_key_usage(&api_key.hash, evaluations)
            .await
            .map_err(|e| match e {
                storage::B2AStorageError::InsufficientFunds => (
                    StatusCode::PAYMENT_REQUIRED,
                    json_error(
                        "monthly_limit_exceeded",
                        "Monthly free tier limit exceeded",
                        &req_id,
                    ),
                ),
                storage::B2AStorageError::Other(msg) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json_error("internal_error", &msg, &req_id),
                ),
            })?;
        tracing::info!(
            request_id = %req_id,
            route = "/v1/evaluate",
            auth_mode = "api_key",
            owner = %record.owner,
            api_key_hash_prefix = %&api_key.hash[..8],
            evaluations = evaluations,
            "api key evaluate request"
        );
        None
    } else {
        let message = payload.message.as_deref().ok_or((
            StatusCode::UNAUTHORIZED,
            json_error(
                "missing_bearer_token",
                "Missing SIWE message or Bearer Token",
                &req_id,
            ),
        ))?;
        let signature = payload.signature.as_deref().ok_or((
            StatusCode::UNAUTHORIZED,
            json_error("missing_bearer_token", "Missing SIWE signature", &req_id),
        ))?;
        let expected_domain = std::env::var("SIWE_DOMAIN").ok();
        let expected_uri = std::env::var("SIWE_URI").ok();
        let (wallet_address, siwe_msg) = verify_siwe(
            message,
            signature,
            expected_domain.as_deref(),
            expected_uri.as_deref(),
        )
        .await
        .map_err(|e| {
            (
                StatusCode::UNAUTHORIZED,
                json_error("unauthorized", &e.to_string(), &req_id),
            )
        })?;
        let nonce_valid = state
            .storage
            .check_and_record_nonce(&wallet_address, siwe_msg.nonce.as_str())
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json_error("internal_error", &e.to_string(), &req_id),
                )
            })?;
        if !nonce_valid {
            return Err((
                StatusCode::UNAUTHORIZED,
                json_error(
                    "unauthorized",
                    "Nonce already used (replay attack detected)",
                    &req_id,
                ),
            ));
        }
        let rate_ok = state
            .storage
            .check_and_record_request(&wallet_address, 60, 10)
            .await
            .unwrap_or(true);
        if !rate_ok {
            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                json_error("rate_limit_exceeded", "Rate limit exceeded", &req_id),
            ));
        }
        Some(wallet_address)
    };

    let result_value = if contexts.len() == 1 {
        rule.evaluate(&contexts[0]).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error(
                    "evaluation_failed",
                    &format!("Evaluation failed: {:?}", e),
                    &req_id,
                ),
            )
        })?
    } else {
        serde_json::Value::Array(rule.evaluate_batch(&contexts).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error(
                    "evaluation_failed",
                    &format!("Evaluation failed: {:?}", e),
                    &req_id,
                ),
            )
        })?)
    };

    if let Some(wallet_address) = wallet_address {
        state
            .storage
            .deduct_balance(&wallet_address, cost)
            .await
            .map_err(|e| match e {
                storage::B2AStorageError::InsufficientFunds => (
                    StatusCode::PAYMENT_REQUIRED,
                    json_error("insufficient_funds", "Insufficient funds", &req_id),
                ),
                storage::B2AStorageError::Other(msg) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json_error("internal_error", &msg, &req_id),
                ),
            })?;
    }

    tracing::info!(
        request_id = %req_id,
        route = "/v1/evaluate",
        status = 200,
        "evaluate request completed"
    );
    Ok((
        StatusCode::OK,
        Json(EvaluateResponse {
            result: result_value,
            cost: cost.to_string(),
        }),
    ))
}

use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

pub fn create_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health))
        .route("/v1/usage", get(usage))
        .route("/v1/estimate", post(estimate))
        .route("/v1/evaluate", post(evaluate))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors)
                .layer(RequestBodyLimitLayer::new(1024 * 1024)), // 1MB limit
        )
        .with_state(state)
}
