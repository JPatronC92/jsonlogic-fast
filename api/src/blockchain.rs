use alloy::network::EthereumWallet;
use alloy::providers::ProviderBuilder;
use alloy::signers::local::PrivateKeySigner;
use alloy::sol;
use std::env;
use std::sync::Arc;
use std::time::Duration;

// Added for runtime secret fetch (no TF injection)
use aws_config;
use aws_sdk_secretsmanager::Client as SecretsManagerClient;

sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    contract B2AStaking {
        mapping(address => uint256) public balances;
        event Deposited(address indexed user, uint256 amount);
        event Slashed(address indexed user, uint256 amount);
        function deposit() public payable;
        function slash(address user, uint256 amount) public;
    }
}

pub struct BlockchainConfig {
    pub rpc_url: String,
    pub private_key: String,
    pub contract_address: alloy::primitives::Address,
}

impl BlockchainConfig {
    /// Sync fallback for local/dev when PRIVATE_KEY is explicitly set in env.
    pub fn from_env() -> Self {
        // Default to Base Sepolia for dev/test. For production (Fase 4) set RPC_URL=https://mainnet.base.org
        // CONTRACT_ADDRESS must be the deployed B2AStaking on Base Mainnet.
        // IMPORTANT: For production use from_env_with_secret_fallback() so PRIVATE_KEY is never required in Lambda env.
        let rpc_url =
            env::var("RPC_URL").unwrap_or_else(|_| "https://sepolia.base.org".to_string());
        let private_key = env::var("PRIVATE_KEY").unwrap_or_else(|_| {
            "0000000000000000000000000000000000000000000000000000000000000001".to_string()
        });
        let contract_address_str = env::var("CONTRACT_ADDRESS")
            .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string());

        let contract_address = contract_address_str
            .parse()
            .expect("Invalid contract address");

        Self {
            rpc_url,
            private_key,
            contract_address,
        }
    }

    /// Preferred for slasher Lambda: falls back to runtime fetch from Secrets Manager.
    /// Secret name: b2a/slasher-private-key-<ENVIRONMENT or dev>
    pub async fn from_env_with_secret_fallback() -> Self {
        let rpc_url =
            env::var("RPC_URL").unwrap_or_else(|_| "https://sepolia.base.org".to_string());
        let contract_address_str = env::var("CONTRACT_ADDRESS")
            .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string());
        let contract_address = contract_address_str
            .parse()
            .expect("Invalid contract address");

        let private_key = resolve_private_key().await;

        Self {
            rpc_url,
            private_key,
            contract_address,
        }
    }

    pub fn read_provider(&self) -> Arc<impl alloy::providers::Provider> {
        Arc::new(ProviderBuilder::new().connect_http(self.rpc_url.parse().unwrap()))
    }

    pub fn write_provider(&self) -> Arc<impl alloy::providers::Provider> {
        let signer: PrivateKeySigner = self.private_key.parse().expect("Invalid private key");
        let wallet = EthereumWallet::from(signer);
        Arc::new(
            ProviderBuilder::new()
                .wallet(wallet)
                .connect_http(self.rpc_url.parse().unwrap()),
        )
    }
}

/// Resolve private key preferring explicit PRIVATE_KEY env (local/dev), else fetch from Secrets Manager at runtime.
/// This removes the need to inject secrets into Lambda environment variables at deploy time.
async fn resolve_private_key() -> String {
    if let Ok(k) = env::var("PRIVATE_KEY") {
        let trimmed = k.trim();
        if !trimmed.is_empty()
            && trimmed != "0000000000000000000000000000000000000000000000000000000000000001"
        {
            return trimmed.to_string();
        }
    }

    // Runtime fetch from Secrets Manager (requires IAM GetSecretValue for b2a/slasher-... )
    let aws_config = aws_config::load_from_env().await;
    let sm_client = SecretsManagerClient::new(&aws_config);

    let env_name = env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string());
    let secret_id = env::var("SLASHER_KEY_SECRET")
        .unwrap_or_else(|_| format!("b2a/slasher-private-key-{}", env_name));

    match sm_client
        .get_secret_value()
        .secret_id(secret_id.clone())
        .send()
        .await
    {
        Ok(out) => {
            if let Some(s) = out.secret_string {
                return s;
            }
            // fallback to secret_binary if base64, but we use string
            eprintln!("Secret {} had no secret_string", secret_id);
        }
        Err(e) => {
            eprintln!(
                "Failed loading secret {} from SecretsManager: {:?}",
                secret_id, e
            );
        }
    }

    if env_name == "prod" {
        panic!("Failed to load real private key in production environment.");
    }

    // last resort dummy (will fail on chain unless test)
    "0000000000000000000000000000000000000000000000000000000000000001".to_string()
}

/// Simple exponential backoff retry wrapper for fallible async operations.
/// Keeps the operation as a closure so it is pure and testable (e.g. with a mock that fails N times).
/// max_attempts: total tries including first.
/// base_delay_ms: initial delay, doubles each retry (capped).
pub async fn retry_with_backoff<F, Fut, T, E>(
    mut operation: F,
    max_attempts: u32,
    base_delay_ms: u64,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut last_err: Option<E> = None;
    for attempt in 1..=max_attempts {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(e) => {
                last_err = Some(e);
                if attempt == max_attempts {
                    break;
                }
                let delay = base_delay_ms * (1u64 << (attempt - 1).min(4)); // 2^0 to 2^4 cap ~16x
                tokio::time::sleep(Duration::from_millis(delay)).await;
            }
        }
    }
    // SAFETY: loop runs at least once so last_err is Some
    Err(last_err.expect("retry_with_backoff: no error recorded"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn retry_succeeds_after_transient_failures() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let result: Result<i32, &str> = retry_with_backoff(
            move || {
                let a = attempts_clone.clone();
                async move {
                    let n = a.fetch_add(1, Ordering::SeqCst) + 1;
                    if n < 3 {
                        Err("transient")
                    } else {
                        Ok(42)
                    }
                }
            },
            5,
            1, // fast
        )
        .await;

        assert_eq!(result, Ok(42));
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn retry_fails_after_max_attempts() {
        let result: Result<(), &str> =
            retry_with_backoff(|| async { Err("always fail") }, 2, 1).await;
        assert_eq!(result, Err("always fail"));
    }
}
