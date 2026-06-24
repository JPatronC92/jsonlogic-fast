use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use aws_sdk_dynamodb::{Client, types::AttributeValue};
use std::time::{SystemTime, UNIX_EPOCH};
use alloy_primitives::U256;

// Minimal proper error type (thiserror) to replace pervasive String errors in Storage impls.
#[derive(thiserror::Error, Debug, serde::Serialize)]
pub enum B2AStorageError {
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("storage error: {0}")]
    Other(String),
}

impl From<B2AStorageError> for String {
    fn from(e: B2AStorageError) -> Self { e.to_string() }
}

#[derive(Clone)]
pub enum StorageBackend {
    Memory(MemoryStorage),
    Dynamo(DynamoStorage),
}

impl StorageBackend {
    pub async fn check_and_record_nonce(&self, address: &str, nonce: &str) -> Result<bool, B2AStorageError> {
        match self {
            StorageBackend::Memory(m) => m.check_and_record_nonce(address, nonce).await,
            StorageBackend::Dynamo(d) => d.check_and_record_nonce(address, nonce).await,
        }
    }

    pub async fn get_balance(&self, address: &str) -> Result<U256, B2AStorageError> {
        match self {
            StorageBackend::Memory(m) => m.get_balance(address).await,
            StorageBackend::Dynamo(d) => d.get_balance(address).await,
        }
    }

    pub async fn deduct_balance(&self, address: &str, cost: U256) -> Result<(), B2AStorageError> {
        match self {
            StorageBackend::Memory(m) => m.deduct_balance(address, cost).await,
            StorageBackend::Dynamo(d) => d.deduct_balance(address, cost).await,
        }
    }

    pub async fn add_balance(&self, address: &str, amount: U256) -> Result<(), B2AStorageError> {
        match self {
            StorageBackend::Memory(m) => m.add_balance(address, amount).await,
            StorageBackend::Dynamo(d) => d.add_balance(address, amount).await,
        }
    }

    pub async fn get_all_balances(&self) -> Result<Vec<(String, U256)>, B2AStorageError> {
        match self {
            StorageBackend::Memory(m) => m.get_all_balances().await,
            StorageBackend::Dynamo(d) => d.get_all_balances().await,
        }
    }

    /// Obtiene el ├║ltimo bloque procesado por el sync (para reanudar sin perder eventos).
    pub async fn get_last_sync_block(&self) -> Result<u64, B2AStorageError> {
        match self {
            StorageBackend::Memory(m) => m.get_last_sync_block().await,
            StorageBackend::Dynamo(d) => d.get_last_sync_block().await,
        }
    }

    pub async fn set_last_sync_block(&self, block: u64) -> Result<(), B2AStorageError> {
        match self {
            StorageBackend::Memory(m) => m.set_last_sync_block(block).await,
            StorageBackend::Dynamo(d) => d.set_last_sync_block(block).await,
        }
    }

    /// Basic rate limit: returns true if allowed (within window)
    pub async fn check_and_record_request(&self, address: &str, window_secs: u64, max_requests: u32) -> Result<bool, B2AStorageError> {
        match self {
            StorageBackend::Memory(m) => m.check_and_record_request(address, window_secs, max_requests).await,
            StorageBackend::Dynamo(d) => d.check_and_record_request(address, window_secs, max_requests).await,
        }
    }
}

#[derive(Clone)]
pub struct MemoryStorage {
    balances: Arc<Mutex<HashMap<String, U256>>>,
    // Nonce -> timestamp (unix seconds) for cleanup, matching Dynamo TTL behavior
    nonces: Arc<Mutex<HashMap<String, u64>>>,
    last_sync_block: Arc<Mutex<u64>>,
    // address -> vec of recent request timestamps (within window) for per-address rate limiting
    last_requests: Arc<Mutex<HashMap<String, Vec<u64>>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            balances: Arc::new(Mutex::new(HashMap::new())),
            nonces: Arc::new(Mutex::new(HashMap::new())),
            last_sync_block: Arc::new(Mutex::new(0)),
            last_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl MemoryStorage {
    pub async fn check_and_record_nonce(&self, address: &str, nonce: &str) -> Result<bool, B2AStorageError> {
        let mut nonces = self.nonces.lock().await;
        let pk = format!("{}#{}", address, nonce);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let ttl = 3600u64;
        let allowed = crate::hardening::nonce_allow(now, ttl, &pk, &mut nonces);
        Ok(allowed)
    }

    pub async fn get_balance(&self, address: &str) -> Result<U256, B2AStorageError> {
        let balances = self.balances.lock().await;
        // Use pure default (ZERO, no 10-unit seed)
        Ok(*balances.get(address).unwrap_or(&crate::hardening::default_balance()))
    }

    pub async fn deduct_balance(&self, address: &str, cost: U256) -> Result<(), B2AStorageError> {
        let mut balances = self.balances.lock().await;
        let current = *balances.get(address).unwrap_or(&U256::ZERO);
        if current < cost {
            return Err(B2AStorageError::InsufficientFunds.into());
        }
        balances.insert(address.to_string(), current - cost);
        Ok(())
    }

    pub async fn add_balance(&self, address: &str, amount: U256) -> Result<(), B2AStorageError> {
        let mut balances = self.balances.lock().await;
        let current = *balances.get(address).unwrap_or(&U256::ZERO);
        balances.insert(address.to_string(), current + amount);
        Ok(())
    }

    pub async fn get_all_balances(&self) -> Result<Vec<(String, U256)>, B2AStorageError> {
        let cache = self.balances.lock().await;
        let mut results = Vec::new();
        for (k, v) in cache.iter() {
            results.push((k.clone(), *v));
        }
        Ok(results)
    }

    pub async fn get_last_sync_block(&self) -> Result<u64, B2AStorageError> {
        let block = self.last_sync_block.lock().await;
        Ok(*block)
    }

    pub async fn set_last_sync_block(&self, block: u64) -> Result<(), B2AStorageError> {
        let mut last = self.last_sync_block.lock().await;
        *last = block;
        Ok(())
    }

    pub async fn check_and_record_request(&self, address: &str, window_secs: u64, max_requests: u32) -> Result<bool, B2AStorageError> {
        let mut requests = self.last_requests.lock().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let entry = requests.entry(address.to_string()).or_insert_with(Vec::new);
        let allowed = crate::hardening::rate_limit_allow(now, window_secs, max_requests, entry);
        Ok(allowed)
    }
}

#[derive(Clone)]
pub struct DynamoStorage {
    client: Client,
    balances_table: String,
    nonces_table: String,
}

impl DynamoStorage {
    pub async fn new(balances_table: &str, nonces_table: &str) -> Self {
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);
        Self {
            client,
            balances_table: balances_table.to_string(),
            nonces_table: nonces_table.to_string(),
        }
    }
}

impl DynamoStorage {
    pub async fn check_and_record_nonce(&self, address: &str, nonce: &str) -> Result<bool, B2AStorageError> {
        // Dynamo uses native put+condition_not_exists + TTL for atomic replay prevention (equivalent to nonce_allow pure logic used by Memory; DB handles cleanup).
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let ttl = now + 3600; // 1 hour TTL

        let pk = format!("{}#{}", address, nonce);

        let result = self.client.put_item()
            .table_name(&self.nonces_table)
            .item("id", AttributeValue::S(pk))
            .item("ttl", AttributeValue::N(ttl.to_string()))
            .condition_expression("attribute_not_exists(id)")
            .send()
            .await;

        match result {
            Ok(_) => Ok(true),
            Err(e) => {
                crate::hardening::interpret_nonce_put(&format!("{:?}", e)).map_err(B2AStorageError::Other)
            }
        }
    }

    pub async fn get_balance(&self, address: &str) -> Result<U256, B2AStorageError> {
        let result = self.client.get_item()
            .table_name(&self.balances_table)
            .key("address", AttributeValue::S(address.to_string()))
            .send()
            .await
            .map_err(|e| B2AStorageError::Other(format!("DynamoDB get error: {:?}", e)))?;

        // thin wrapper: use adapter for decision
        Ok(crate::hardening::balance_from_item(result.item.as_ref()))
    }

    pub async fn deduct_balance(&self, address: &str, cost: U256) -> Result<(), B2AStorageError> {
        // Optimistic locking loop para atomicidad
        loop {
            let current = self.get_balance(address).await?;
            if current < cost {
                return Err(B2AStorageError::InsufficientFunds);
            }
            let new_balance = current - cost;

            let result = self
                .client
                .update_item()
                .table_name(&self.balances_table)
                .key("address", AttributeValue::S(address.to_string()))
                .update_expression("SET balance = :new_balance")
                .condition_expression("balance = :current_balance")
                .expression_attribute_values(
                    ":current_balance",
                    AttributeValue::S(current.to_string()),
                )
                .expression_attribute_values(
                    ":new_balance",
                    AttributeValue::S(new_balance.to_string()),
                )
                .send()
                .await;

            match result {
                Ok(_) => return Ok(()),
                Err(e) => {
                    let err_str = format!("{:?}", e);
                    if err_str.contains("ConditionalCheckFailedException") {
                        // Conflicto de concurrencia, reintentar
                        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                        continue;
                    } else {
                        return Err(B2AStorageError::Other(format!("DynamoDB deduct error: {}", err_str)));
                    }
                }
            }
        }
    }

    pub async fn add_balance(&self, address: &str, amount: U256) -> Result<(), B2AStorageError> {
        // Usamos get + conditional put para consistencia (aunque adds son menos cr├¡ticos)
        loop {
            let current = self.get_balance(address).await.unwrap_or(U256::ZERO);
            let new_balance = current + amount;

            let result = self
                .client
                .update_item()
                .table_name(&self.balances_table)
                .key("address", AttributeValue::S(address.to_string()))
                .update_expression("SET balance = :new_balance")
                .condition_expression("balance = :current_balance")
                .expression_attribute_values(
                    ":current_balance",
                    AttributeValue::S(current.to_string()),
                )
                .expression_attribute_values(
                    ":new_balance",
                    AttributeValue::S(new_balance.to_string()),
                )
                .send()
                .await;

            match result {
                Ok(_) => return Ok(()),
                Err(e) => {
                    let err_str = format!("{:?}", e);
                    if err_str.contains("ConditionalCheckFailedException") {
                        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                        continue;
                    } else {
                        return Err(B2AStorageError::Other(format!("DynamoDB add error: {}", err_str)));
                    }
                }
            }
        }
    }

    pub async fn get_all_balances(&self) -> Result<Vec<(String, U256)>, B2AStorageError> {
        // Use FilterExpression to exclude internal __meta and __rate keys (avoids full scan cost for slasher)
        let mut scan = self.client.scan()
            .table_name(&self.balances_table)
            .filter_expression("NOT begins_with(#addr, :p)")
            .expression_attribute_names("#addr", "address")
            .expression_attribute_values(":p", AttributeValue::S("__".to_string()))
            .into_paginator()
            .send();

        let mut results = Vec::new();
        
        while let Some(result) = scan.next().await {
            match result {
                Ok(output) => {
                    if let Some(items) = output.items {
                        for item in items {
                            if let (Some(id_val), Some(bal_val)) = (item.get("address"), item.get("balance")) {
                                if let Ok(id) = id_val.as_s() {
                                    if !crate::hardening::include_user_balance_key(id) { continue; }
                                    let bal = if let Ok(bal_str) = bal_val.as_s() {
                                        U256::from_str(bal_str).unwrap_or(U256::ZERO)
                                    } else if let Ok(bal_str) = bal_val.as_n() {
                                        bal_str.parse::<f64>().map(|f| U256::from((f * 1e18) as u128)).unwrap_or(U256::ZERO)
                                    } else {
                                        U256::ZERO
                                    };
                                    results.push((id.clone(), bal));
                                }
                            }
                        }
                    }
                }
                Err(e) => return Err(B2AStorageError::Other(format!("DynamoDB scan failed: {:?}", e))),
            }
        }
        Ok(results)
    }

    pub async fn get_last_sync_block(&self) -> Result<u64, B2AStorageError> {
        let result = self.client.get_item()
            .table_name(&self.balances_table)
            .key("address", AttributeValue::S("__meta:last_sync_block".to_string()))
            .send()
            .await
            .map_err(|e| B2AStorageError::Other(format!("DynamoDB get last block error: {:?}", e)))?;

        if let Some(item) = result.item {
            if let Some(AttributeValue::S(block_str)) = item.get("balance") {
                return block_str.parse::<u64>().map_err(|_| B2AStorageError::Other("Invalid block format".to_string()));
            }
        }
        Ok(0) // default: start from genesis or 0, caller can decide
    }

    pub async fn set_last_sync_block(&self, block: u64) -> Result<(), B2AStorageError> {
        self.client.put_item()
            .table_name(&self.balances_table)
            .item("address", AttributeValue::S("__meta:last_sync_block".to_string()))
            .item("balance", AttributeValue::S(block.to_string()))
            .send()
            .await
            .map_err(|e| B2AStorageError::Other(format!("DynamoDB set last block error: {:?}", e)))?;

        Ok(())
    }

    pub async fn check_and_record_request(&self, address: &str, window_secs: u64, max_requests: u32) -> Result<bool, B2AStorageError> {
        // Atomic via optimistic locking + ConditionExpression (mirrors deduct_balance).
        // Re-get + re-decide on conflict. Pure decide in hardening is unchanged.
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let key = format!("__rate:{}", address);

        loop {
            let mut timestamps: Vec<u64> = Vec::new();
            let mut read_serialized = String::new();
            if let Ok(result) = self.client.get_item()
                .table_name(&self.balances_table)
                .key("address", AttributeValue::S(key.clone()))
                .send()
                .await
            {
                if let Some(item) = result.item {
                    if let Some(AttributeValue::S(list_str)) = item.get("rate_ts") {
                        read_serialized = list_str.clone();
                        timestamps = crate::hardening::parse_rate_ts(list_str);
                    }
                }
            }

            let allowed = crate::hardening::dynamo_rate_decide(now, window_secs, max_requests, &mut timestamps);

            if !allowed {
                return Ok(false);
            }

            let new_list_str = crate::hardening::serialize_rate_ts(&timestamps);

            // Use extracted pure helper for condition
            let (cond_expr, maybe_val) = crate::hardening::rate_put_condition(&read_serialized);
            let mut put_req = self.client.put_item()
                .table_name(&self.balances_table)
                .item("address", AttributeValue::S(key.clone()))
                .item("rate_ts", AttributeValue::S(new_list_str.clone()))
                .condition_expression(cond_expr);
            if let Some((k, v)) = maybe_val {
                put_req = put_req.expression_attribute_values(&k, v);
            }

            match put_req.send().await {
                Ok(_) => return Ok(true),
                Err(e) => {
                    let err_str = format!("{:?}", e);
                    if err_str.contains("ConditionalCheckFailedException") {
                        // concurrent change, retry
                        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                        continue;
                    } else {
                        return Err(B2AStorageError::Other(format!("Dynamo rate conditional put failed: {}", err_str)));
                    }
                }
            }
        }
    }
}
