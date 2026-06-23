use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use aws_sdk_dynamodb::{Client, types::AttributeValue};
use std::time::{SystemTime, UNIX_EPOCH};
use alloy_primitives::U256;

#[derive(Clone)]
pub enum StorageBackend {
    Memory(MemoryStorage),
    Dynamo(DynamoStorage),
}

impl StorageBackend {
    pub async fn check_and_record_nonce(&self, address: &str, nonce: &str) -> Result<bool, String> {
        match self {
            StorageBackend::Memory(m) => m.check_and_record_nonce(address, nonce).await,
            StorageBackend::Dynamo(d) => d.check_and_record_nonce(address, nonce).await,
        }
    }

    pub async fn get_balance(&self, address: &str) -> Result<U256, String> {
        match self {
            StorageBackend::Memory(m) => m.get_balance(address).await,
            StorageBackend::Dynamo(d) => d.get_balance(address).await,
        }
    }

    pub async fn deduct_balance(&self, address: &str, cost: U256) -> Result<(), String> {
        match self {
            StorageBackend::Memory(m) => m.deduct_balance(address, cost).await,
            StorageBackend::Dynamo(d) => d.deduct_balance(address, cost).await,
        }
    }

    pub async fn add_balance(&self, address: &str, amount: U256) -> Result<(), String> {
        match self {
            StorageBackend::Memory(m) => m.add_balance(address, amount).await,
            StorageBackend::Dynamo(d) => d.add_balance(address, amount).await,
        }
    }

    pub async fn get_all_balances(&self) -> Result<Vec<(String, U256)>, String> {
        match self {
            StorageBackend::Memory(m) => m.get_all_balances().await,
            StorageBackend::Dynamo(d) => d.get_all_balances().await,
        }
    }

    /// Obtiene el último bloque procesado por el sync (para reanudar sin perder eventos).
    pub async fn get_last_sync_block(&self) -> Result<u64, String> {
        match self {
            StorageBackend::Memory(m) => m.get_last_sync_block().await,
            StorageBackend::Dynamo(d) => d.get_last_sync_block().await,
        }
    }

    pub async fn set_last_sync_block(&self, block: u64) -> Result<(), String> {
        match self {
            StorageBackend::Memory(m) => m.set_last_sync_block(block).await,
            StorageBackend::Dynamo(d) => d.set_last_sync_block(block).await,
        }
    }

    /// Basic rate limit: returns true if allowed (within window)
    pub async fn check_and_record_request(&self, address: &str, window_secs: u64, max_requests: u32) -> Result<bool, String> {
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
    // address -> last request timestamp for basic rate limiting
    last_requests: Arc<Mutex<HashMap<String, u64>>>,
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

/// Default seed balance: 10 units with 18 decimals


impl MemoryStorage {
    pub async fn check_and_record_nonce(&self, address: &str, nonce: &str) -> Result<bool, String> {
        let mut nonces = self.nonces.lock().await;
        let pk = format!("{}#{}", address, nonce);

        // Cleanup old nonces (TTL 1h like Dynamo) to prevent memory leak
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let ttl = 3600u64;
        nonces.retain(|_, ts| now - *ts < ttl);

        if nonces.contains_key(&pk) {
            Ok(false)
        } else {
            nonces.insert(pk, now);
            Ok(true)
        }
    }

    pub async fn get_balance(&self, address: &str) -> Result<U256, String> {
        let balances = self.balances.lock().await;
        // No default seed balance - only what has been deposited/added
        Ok(*balances.get(address).unwrap_or(&U256::ZERO))
    }

    pub async fn deduct_balance(&self, address: &str, cost: U256) -> Result<(), String> {
        let mut balances = self.balances.lock().await;
        let current = *balances.get(address).unwrap_or(&U256::ZERO);
        if current < cost {
            return Err("Insufficient funds".to_string());
        }
        balances.insert(address.to_string(), current - cost);
        Ok(())
    }

    pub async fn add_balance(&self, address: &str, amount: U256) -> Result<(), String> {
        let mut balances = self.balances.lock().await;
        let current = *balances.get(address).unwrap_or(&U256::ZERO);
        balances.insert(address.to_string(), current + amount);
        Ok(())
    }

    pub async fn get_all_balances(&self) -> Result<Vec<(String, U256)>, String> {
        let cache = self.balances.lock().await;
        let mut results = Vec::new();
        for (k, v) in cache.iter() {
            results.push((k.clone(), *v));
        }
        Ok(results)
    }

    pub async fn get_last_sync_block(&self) -> Result<u64, String> {
        let block = self.last_sync_block.lock().await;
        Ok(*block)
    }

    pub async fn set_last_sync_block(&self, block: u64) -> Result<(), String> {
        let mut last = self.last_sync_block.lock().await;
        *last = block;
        Ok(())
    }

    pub async fn check_and_record_request(&self, address: &str, window_secs: u64, max_requests: u32) -> Result<bool, String> {
        let mut requests = self.last_requests.lock().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Clean old
        requests.retain(|_, ts| now - *ts < window_secs);

        let count = requests.values().filter(|&&ts| now - ts < window_secs).count() as u32; // simplistic count in window

        if count >= max_requests {
            return Ok(false);
        }

        requests.insert(address.to_string(), now);
        Ok(true)
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
    pub async fn check_and_record_nonce(&self, address: &str, nonce: &str) -> Result<bool, String> {
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
                let err_msg = format!("{:?}", e);
                if err_msg.contains("ConditionalCheckFailedException") {
                    Ok(false)
                } else {
                    Err(format!("DynamoDB error: {}", err_msg))
                }
            }
        }
    }

    pub async fn get_balance(&self, address: &str) -> Result<U256, String> {
        let result = self.client.get_item()
            .table_name(&self.balances_table)
            .key("address", AttributeValue::S(address.to_string()))
            .send()
            .await
            .map_err(|e| format!("DynamoDB get error: {:?}", e))?;

        if let Some(item) = result.item {
            if let Some(AttributeValue::S(bal_str)) = item.get("balance") {
                return U256::from_str(bal_str).map_err(|_| "Invalid balance format".to_string());
            }
            // legacy support if old N values exist
            if let Some(AttributeValue::N(bal_str)) = item.get("balance") {
                if let Ok(f) = bal_str.parse::<f64>() {
                    return Ok(U256::from((f * 1e18) as u128));
                }
            }
        }
        
        // No default seed - credit only comes from on-chain deposits via sync
        Ok(U256::ZERO)
    }

    pub async fn deduct_balance(&self, address: &str, cost: U256) -> Result<(), String> {
        // Optimistic locking loop para atomicidad
        loop {
            let current = self.get_balance(address).await?;
            if current < cost {
                return Err("Insufficient funds".to_string());
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
                        return Err(format!("DynamoDB deduct error: {}", err_str));
                    }
                }
            }
        }
    }

    pub async fn add_balance(&self, address: &str, amount: U256) -> Result<(), String> {
        // Usamos get + conditional put para consistencia (aunque adds son menos críticos)
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
                        return Err(format!("DynamoDB add error: {}", err_str));
                    }
                }
            }
        }
    }

    pub async fn get_all_balances(&self) -> Result<Vec<(String, U256)>, String> {
        let mut scan = self.client.scan().table_name(&self.balances_table).into_paginator().send();
        let mut results = Vec::new();
        
        while let Some(result) = scan.next().await {
            match result {
                Ok(output) => {
                    if let Some(items) = output.items {
                        for item in items {
                            if let (Some(id_val), Some(bal_val)) = (item.get("address"), item.get("balance")) {
                                if let Ok(id) = id_val.as_s() {
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
                Err(e) => return Err(format!("DynamoDB scan failed: {:?}", e)),
            }
        }
        Ok(results)
    }

    pub async fn get_last_sync_block(&self) -> Result<u64, String> {
        let result = self.client.get_item()
            .table_name(&self.balances_table)
            .key("address", AttributeValue::S("__meta:last_sync_block".to_string()))
            .send()
            .await
            .map_err(|e| format!("DynamoDB get last block error: {:?}", e))?;

        if let Some(item) = result.item {
            if let Some(AttributeValue::S(block_str)) = item.get("balance") {
                return block_str.parse::<u64>().map_err(|_| "Invalid block format".to_string());
            }
        }
        Ok(0) // default: start from genesis or 0, caller can decide
    }

    pub async fn set_last_sync_block(&self, block: u64) -> Result<(), String> {
        self.client.put_item()
            .table_name(&self.balances_table)
            .item("address", AttributeValue::S("__meta:last_sync_block".to_string()))
            .item("balance", AttributeValue::S(block.to_string()))
            .send()
            .await
            .map_err(|e| format!("DynamoDB set last block error: {:?}", e))?;

        Ok(())
    }

    pub async fn check_and_record_request(&self, address: &str, window_secs: u64, max_requests: u32) -> Result<bool, String> {
        // Simple last-request based rate limit for Dynamo (full counter would use separate table or GSI)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let key = format!("__rate:last:{}", address);

        let result = self.client.get_item()
            .table_name(&self.balances_table)
            .key("address", AttributeValue::S(key.clone()))
            .send()
            .await
            .map_err(|e| format!("Dynamo rate get error: {:?}", e))?;

        if let Some(item) = result.item {
            if let Some(AttributeValue::S(ts_str)) = item.get("balance") {
                if let Ok(last_ts) = ts_str.parse::<u64>() {
                    if now - last_ts < window_secs {
                        // For simplicity, allow 1 per window; for real use counter
                        return Ok(false);
                    }
                }
            }
        }

        // Record
        self.client.put_item()
            .table_name(&self.balances_table)
            .item("address", AttributeValue::S(key))
            .item("balance", AttributeValue::S(now.to_string()))
            .send()
            .await
            .map_err(|e| format!("Dynamo rate set error: {:?}", e))?;

        Ok(true)
    }
}
