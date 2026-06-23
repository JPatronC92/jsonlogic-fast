use api::{create_app, AppState};
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use http_body_util::BodyExt;
use k256::ecdsa::SigningKey;
use serde_json::{json, Value};
use sha3::{Digest, Keccak256};
use std::sync::Arc;
use time::OffsetDateTime;
use tower::ServiceExt;
use api::storage::{StorageBackend, MemoryStorage};

fn to_eip55_address(address_bytes: &[u8; 20]) -> String {
    let address_hex = hex::encode(address_bytes); // lowercase hex
    let hash = Keccak256::digest(address_hex.as_bytes());
    let hash_hex = hex::encode(hash);

    let mut eip55 = String::with_capacity(42);
    eip55.push_str("0x");

    for (i, c) in address_hex.chars().enumerate() {
        let hash_char = hash_hex.as_bytes()[i];
        if hash_char >= b'8' {
            eip55.push(c.to_ascii_uppercase());
        } else {
            eip55.push(c);
        }
    }

    eip55
}

fn generate_wallet() -> (SigningKey, String) {
    let signing_key = SigningKey::random(&mut rand::rngs::OsRng);
    let verifying_key = signing_key.verifying_key();
    let public_key_bytes = verifying_key.to_encoded_point(false);
    let hash = Keccak256::digest(&public_key_bytes.as_bytes()[1..]);
    let mut address_bytes = [0u8; 20];
    address_bytes.copy_from_slice(&hash[12..]); // last 20 bytes
    let address_eip55 = to_eip55_address(&address_bytes);
    (signing_key, address_eip55)
}

fn sign_siwe_message(signing_key: &SigningKey, message: &str) -> String {
    let eth_message = format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message);
    let eth_message_hash = Keccak256::digest(eth_message.as_bytes());

    // Sign using ecdsa recovery
    let (signature, recovery_id) = signing_key
        .sign_prehash_recoverable(&eth_message_hash)
        .unwrap();

    let mut sig_bytes = [0u8; 65];
    sig_bytes[..64].copy_from_slice(&signature.to_bytes());
    sig_bytes[64] = recovery_id.to_byte() + 27;

    format!("0x{}", hex::encode(sig_bytes))
}

#[tokio::test]
async fn test_estimate_endpoint() {
    let state = AppState {
        storage: Arc::new(StorageBackend::Memory(MemoryStorage::new())),
    };
    let app = create_app(state);

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/v1/estimate")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "rule_depth": 3,
                        "batch_size": 10
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let res_json: Value = serde_json::from_slice(&body).unwrap();

    // Expected cost = 0.0001 * (1.0 + (3 * 0.1)) * 10 = 0.0013
    let diff = (res_json["estimated_cost"].as_f64().unwrap() - 0.0013).abs();
    assert!(diff < 1e-9, "Cost difference {} too large", diff);
}

#[tokio::test]
async fn test_evaluate_endpoint() {
    let (signing_key, address) = generate_wallet();

    let storage = MemoryStorage::new();
    // Seed the wallet balance
    storage.add_balance(&address, 10.0).await.unwrap();

    let state = AppState {
        storage: Arc::new(StorageBackend::Memory(storage)),
    };

    let app = create_app(state);

    // Format current time in UTC ISO 8601
    let now = OffsetDateTime::now_utc();
    let now_str = now
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();

    let siwe_message = format!(
        "localhost:3000 wants you to sign in with your Ethereum account:\n\
         {}\n\n\
         Sign in to jsonlogic-fast B2A Serverless API.\n\n\
         URI: http://localhost:3000/v1/evaluate\n\
         Version: 1\n\
         Chain ID: 1\n\
         Nonce: random-nonce-123456\n\
         Issued At: {}",
        address, now_str
    );

    let signature = sign_siwe_message(&signing_key, &siwe_message);

    // Dynamic depth of {"+": [{"var": "a"}, {"var": "b"}]} is 2.
    // Batch size of data (object) is 1.
    // Cost = 0.0001 * (1.0 + (2 * 0.1)) * 1 = 0.00012.
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/v1/evaluate")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "message": siwe_message,
                        "signature": signature,
                        "rule": { "+": [{"var": "a"}, {"var": "b"}] },
                        "data": { "a": 10, "b": 20 }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8_lossy(&body);
    println!("Response status: {}, body: {}", status, body_str);
    assert_eq!(status, StatusCode::OK);

    let res_json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(res_json["result"].as_f64().unwrap(), 30.0);
    let diff = (res_json["cost"].as_f64().unwrap() - 0.00012).abs();
    assert!(diff < 1e-9);
}
