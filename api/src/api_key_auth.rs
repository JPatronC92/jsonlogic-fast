use axum::http::{header, HeaderMap};
use sha2::{Digest, Sha256};

pub const FREE_TIER_MONTHLY_LIMIT: u64 = 1_000;
pub const FREE_TIER_RATE_LIMIT_PER_MINUTE: u32 = 10;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ApiKeyRecord {
    pub api_key_hash: String,
    pub owner: String,
    pub plan: String,
    pub active: bool,
    pub monthly_limit: u64,
    pub used_this_month: u64,
    pub rate_limit_per_minute: u32,
}

impl ApiKeyRecord {
    pub fn free_tier(api_key: &str, owner: impl Into<String>) -> Self {
        let api_key_hash = hash_api_key(api_key);
        Self {
            api_key_hash,
            owner: owner.into(),
            plan: "free".to_string(),
            active: true,
            monthly_limit: FREE_TIER_MONTHLY_LIMIT,
            used_this_month: 0,
            rate_limit_per_minute: FREE_TIER_RATE_LIMIT_PER_MINUTE,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApiKeyAuth {
    pub hash: String,
}

impl ApiKeyAuth {
    pub fn from_headers(headers: &HeaderMap) -> Option<Self> {
        let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
        let token = value.strip_prefix("Bearer ")?.trim();
        if token.is_empty() {
            return None;
        }
        Some(Self {
            hash: hash_api_key(token),
        })
    }
}

pub fn hash_api_key(api_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    hex::encode(hasher.finalize())
}
