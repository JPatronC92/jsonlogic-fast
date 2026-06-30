use crate::api_key_auth::ApiKeyRecord;
use alloy_primitives::U256;
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

#[derive(thiserror::Error, Debug, serde::Serialize)]
pub enum B2AStorageError {
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("storage error: {0}")]
    Other(String),
}

impl From<B2AStorageError> for String {
    fn from(e: B2AStorageError) -> Self {
        e.to_string()
    }
}

#[derive(Clone)]
pub enum StorageBackend {
    Memory(MemoryStorage),
    Dynamo(DynamoStorage),
}

impl StorageBackend {
    pub async fn check_and_record_nonce(
        &self,
        address: &str,
        nonce: &str,
    ) -> Result<bool, B2AStorageError> {
        match self {
            Self::Memory(m) => m.check_and_record_nonce(address, nonce).await,
            Self::Dynamo(d) => d.check_and_record_nonce(address, nonce).await,
        }
    }
    pub async fn get_balance(&self, address: &str) -> Result<U256, B2AStorageError> {
        match self {
            Self::Memory(m) => m.get_balance(address).await,
            Self::Dynamo(d) => d.get_balance(address).await,
        }
    }
    pub async fn deduct_balance(&self, address: &str, cost: U256) -> Result<(), B2AStorageError> {
        match self {
            Self::Memory(m) => m.deduct_balance(address, cost).await,
            Self::Dynamo(d) => d.deduct_balance(address, cost).await,
        }
    }
    pub async fn add_balance(&self, address: &str, amount: U256) -> Result<(), B2AStorageError> {
        match self {
            Self::Memory(m) => m.add_balance(address, amount).await,
            Self::Dynamo(d) => d.add_balance(address, amount).await,
        }
    }
    pub async fn get_all_balances(&self) -> Result<Vec<(String, U256)>, B2AStorageError> {
        match self {
            Self::Memory(m) => m.get_all_balances().await,
            Self::Dynamo(d) => d.get_all_balances().await,
        }
    }
    pub async fn get_last_sync_block(&self) -> Result<u64, B2AStorageError> {
        match self {
            Self::Memory(m) => m.get_last_sync_block().await,
            Self::Dynamo(d) => d.get_last_sync_block().await,
        }
    }
    pub async fn set_last_sync_block(&self, block: u64) -> Result<(), B2AStorageError> {
        match self {
            Self::Memory(m) => m.set_last_sync_block(block).await,
            Self::Dynamo(d) => d.set_last_sync_block(block).await,
        }
    }
    pub async fn check_and_record_request(
        &self,
        address: &str,
        window_secs: u64,
        max_requests: u32,
    ) -> Result<bool, B2AStorageError> {
        match self {
            Self::Memory(m) => {
                m.check_and_record_request(address, window_secs, max_requests)
                    .await
            }
            Self::Dynamo(d) => {
                d.check_and_record_request(address, window_secs, max_requests)
                    .await
            }
        }
    }
    pub async fn put_api_key(&self, record: ApiKeyRecord) -> Result<(), B2AStorageError> {
        match self {
            Self::Memory(m) => m.put_api_key(record).await,
            Self::Dynamo(d) => d.put_api_key(record).await,
        }
    }
    pub async fn get_api_key(
        &self,
        api_key_hash: &str,
    ) -> Result<Option<ApiKeyRecord>, B2AStorageError> {
        match self {
            Self::Memory(m) => m.get_api_key(api_key_hash).await,
            Self::Dynamo(d) => d.get_api_key(api_key_hash).await,
        }
    }
    pub async fn increment_api_key_usage(
        &self,
        api_key_hash: &str,
        amount: u64,
    ) -> Result<(), B2AStorageError> {
        match self {
            Self::Memory(m) => m.increment_api_key_usage(api_key_hash, amount).await,
            Self::Dynamo(d) => d.increment_api_key_usage(api_key_hash, amount).await,
        }
    }
}

#[derive(Clone)]
pub struct MemoryStorage {
    balances: Arc<Mutex<HashMap<String, U256>>>,
    nonces: Arc<Mutex<HashMap<String, u64>>>,
    last_sync_block: Arc<Mutex<u64>>,
    last_requests: Arc<Mutex<HashMap<String, Vec<u64>>>>,
    api_keys: Arc<Mutex<HashMap<String, ApiKeyRecord>>>,
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            balances: Arc::new(Mutex::new(HashMap::new())),
            nonces: Arc::new(Mutex::new(HashMap::new())),
            last_sync_block: Arc::new(Mutex::new(0)),
            last_requests: Arc::new(Mutex::new(HashMap::new())),
            api_keys: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    pub async fn check_and_record_nonce(
        &self,
        address: &str,
        nonce: &str,
    ) -> Result<bool, B2AStorageError> {
        let mut nonces = self.nonces.lock().await;
        let pk = format!("{}#{}", address, nonce);
        let now = now_secs();
        Ok(crate::hardening::nonce_allow(now, 3600, &pk, &mut nonces))
    }
    pub async fn get_balance(&self, address: &str) -> Result<U256, B2AStorageError> {
        Ok(*self
            .balances
            .lock()
            .await
            .get(address)
            .unwrap_or(&crate::hardening::default_balance()))
    }
    pub async fn deduct_balance(&self, address: &str, cost: U256) -> Result<(), B2AStorageError> {
        let mut balances = self.balances.lock().await;
        let current = *balances.get(address).unwrap_or(&U256::ZERO);
        if current < cost {
            return Err(B2AStorageError::InsufficientFunds);
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
        Ok(self
            .balances
            .lock()
            .await
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect())
    }
    pub async fn get_last_sync_block(&self) -> Result<u64, B2AStorageError> {
        Ok(*self.last_sync_block.lock().await)
    }
    pub async fn set_last_sync_block(&self, block: u64) -> Result<(), B2AStorageError> {
        *self.last_sync_block.lock().await = block;
        Ok(())
    }
    pub async fn check_and_record_request(
        &self,
        address: &str,
        window_secs: u64,
        max_requests: u32,
    ) -> Result<bool, B2AStorageError> {
        let mut requests = self.last_requests.lock().await;
        let entry = requests.entry(address.to_string()).or_insert_with(Vec::new);
        Ok(crate::hardening::rate_limit_allow(
            now_secs(),
            window_secs,
            max_requests,
            entry,
        ))
    }
    pub async fn put_api_key(&self, record: ApiKeyRecord) -> Result<(), B2AStorageError> {
        self.api_keys
            .lock()
            .await
            .insert(record.api_key_hash.clone(), record);
        Ok(())
    }
    pub async fn get_api_key(
        &self,
        api_key_hash: &str,
    ) -> Result<Option<ApiKeyRecord>, B2AStorageError> {
        Ok(self.api_keys.lock().await.get(api_key_hash).cloned())
    }
    pub async fn increment_api_key_usage(
        &self,
        api_key_hash: &str,
        amount: u64,
    ) -> Result<(), B2AStorageError> {
        let mut api_keys = self.api_keys.lock().await;
        let record = api_keys
            .get_mut(api_key_hash)
            .ok_or_else(|| B2AStorageError::Other("API key not found".to_string()))?;
        let next = record.used_this_month.saturating_add(amount);
        if next > record.monthly_limit {
            return Err(B2AStorageError::InsufficientFunds);
        }
        record.used_this_month = next;
        Ok(())
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

    pub async fn check_and_record_nonce(
        &self,
        address: &str,
        nonce: &str,
    ) -> Result<bool, B2AStorageError> {
        let ttl = now_secs() + 3600;
        let pk = format!("{}#{}", address, nonce);
        let result = self
            .client
            .put_item()
            .table_name(&self.nonces_table)
            .item("id", AttributeValue::S(pk))
            .item("ttl", AttributeValue::N(ttl.to_string()))
            .condition_expression("attribute_not_exists(id)")
            .send()
            .await;
        match result {
            Ok(_) => Ok(true),
            Err(e) => crate::hardening::interpret_nonce_put(&format!("{:?}", e))
                .map_err(B2AStorageError::Other),
        }
    }
    pub async fn get_balance(&self, address: &str) -> Result<U256, B2AStorageError> {
        let result = self
            .client
            .get_item()
            .table_name(&self.balances_table)
            .key("address", AttributeValue::S(address.to_string()))
            .send()
            .await
            .map_err(|e| B2AStorageError::Other(format!("DynamoDB get error: {:?}", e)))?;
        Ok(crate::hardening::balance_from_item(result.item.as_ref()))
    }
    pub async fn deduct_balance(&self, address: &str, cost: U256) -> Result<(), B2AStorageError> {
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
                        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                        continue;
                    }
                    return Err(B2AStorageError::Other(format!(
                        "DynamoDB deduct error: {}",
                        err_str
                    )));
                }
            }
        }
    }
    pub async fn add_balance(&self, address: &str, amount: U256) -> Result<(), B2AStorageError> {
        loop {
            let current = self.get_balance(address).await.unwrap_or(U256::ZERO);
            let new_balance = current + amount;
            let result = self
                .client
                .update_item()
                .table_name(&self.balances_table)
                .key("address", AttributeValue::S(address.to_string()))
                .update_expression("SET balance = :new_balance")
                .condition_expression(crate::hardening::dynamo_add_balance_condition())
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
                    }
                    return Err(B2AStorageError::Other(format!(
                        "DynamoDB add error: {}",
                        err_str
                    )));
                }
            }
        }
    }
    pub async fn get_all_balances(&self) -> Result<Vec<(String, U256)>, B2AStorageError> {
        let mut scan = self
            .client
            .scan()
            .table_name(&self.balances_table)
            .filter_expression("NOT begins_with(#addr, :p)")
            .expression_attribute_names("#addr", "address")
            .expression_attribute_values(":p", AttributeValue::S("__".to_string()))
            .into_paginator()
            .send();
        let mut results = Vec::new();
        while let Some(result) = scan.next().await {
            let output = result
                .map_err(|e| B2AStorageError::Other(format!("DynamoDB scan failed: {:?}", e)))?;
            if let Some(items) = output.items {
                for item in items {
                    if let (Some(id_val), Some(bal_val)) =
                        (item.get("address"), item.get("balance"))
                    {
                        if let Ok(id) = id_val.as_s() {
                            if !crate::hardening::include_user_balance_key(id) {
                                continue;
                            }
                            let bal = if let Ok(bal_str) = bal_val.as_s() {
                                U256::from_str(bal_str).unwrap_or(U256::ZERO)
                            } else if let Ok(bal_str) = bal_val.as_n() {
                                U256::from_str(bal_str).unwrap_or(U256::ZERO)
                            } else {
                                U256::ZERO
                            };
                            results.push((id.clone(), bal));
                        }
                    }
                }
            }
        }
        Ok(results)
    }
    pub async fn get_last_sync_block(&self) -> Result<u64, B2AStorageError> {
        let result = self
            .client
            .get_item()
            .table_name(&self.balances_table)
            .key(
                "address",
                AttributeValue::S("__meta:last_sync_block".to_string()),
            )
            .send()
            .await
            .map_err(|e| {
                B2AStorageError::Other(format!("DynamoDB get last block error: {:?}", e))
            })?;
        if let Some(item) = result.item {
            if let Some(AttributeValue::S(block_str)) = item.get("balance") {
                return block_str
                    .parse::<u64>()
                    .map_err(|_| B2AStorageError::Other("Invalid block format".to_string()));
            }
        }
        Ok(0)
    }
    pub async fn set_last_sync_block(&self, block: u64) -> Result<(), B2AStorageError> {
        self.client
            .put_item()
            .table_name(&self.balances_table)
            .item(
                "address",
                AttributeValue::S("__meta:last_sync_block".to_string()),
            )
            .item("balance", AttributeValue::S(block.to_string()))
            .send()
            .await
            .map_err(|e| {
                B2AStorageError::Other(format!("DynamoDB set last block error: {:?}", e))
            })?;
        Ok(())
    }
    pub async fn check_and_record_request(
        &self,
        address: &str,
        window_secs: u64,
        max_requests: u32,
    ) -> Result<bool, B2AStorageError> {
        let now = now_secs();
        let key = format!("__rate:{}", address);
        loop {
            let mut timestamps = Vec::new();
            let mut read_serialized = String::new();
            if let Ok(result) = self
                .client
                .get_item()
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
            if !crate::hardening::dynamo_rate_decide(
                now,
                window_secs,
                max_requests,
                &mut timestamps,
            ) {
                return Ok(false);
            }
            let new_list_str = crate::hardening::serialize_rate_ts(&timestamps);
            let (cond_expr, maybe_val) = crate::hardening::rate_put_condition(&read_serialized);
            let mut put_req = self
                .client
                .put_item()
                .table_name(&self.balances_table)
                .item("address", AttributeValue::S(key.clone()))
                .item("rate_ts", AttributeValue::S(new_list_str))
                .condition_expression(cond_expr);
            if let Some((k, v)) = maybe_val {
                put_req = put_req.expression_attribute_values(&k, v);
            }
            match put_req.send().await {
                Ok(_) => return Ok(true),
                Err(e) => {
                    let err_str = format!("{:?}", e);
                    if err_str.contains("ConditionalCheckFailedException") {
                        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                        continue;
                    }
                    return Err(B2AStorageError::Other(format!(
                        "Dynamo rate conditional put failed: {}",
                        err_str
                    )));
                }
            }
        }
    }
    pub async fn put_api_key(&self, record: ApiKeyRecord) -> Result<(), B2AStorageError> {
        self.client
            .put_item()
            .table_name(&self.balances_table)
            .item(
                "address",
                AttributeValue::S(format!("__api_key:{}", record.api_key_hash)),
            )
            .item(
                "api_key_hash",
                AttributeValue::S(record.api_key_hash.clone()),
            )
            .item("owner", AttributeValue::S(record.owner.clone()))
            .item("plan", AttributeValue::S(record.plan.clone()))
            .item("active", AttributeValue::Bool(record.active))
            .item(
                "monthly_limit",
                AttributeValue::N(record.monthly_limit.to_string()),
            )
            .item(
                "used_this_month",
                AttributeValue::N(record.used_this_month.to_string()),
            )
            .item(
                "rate_limit_per_minute",
                AttributeValue::N(record.rate_limit_per_minute.to_string()),
            )
            .send()
            .await
            .map_err(|e| B2AStorageError::Other(format!("DynamoDB put api key error: {:?}", e)))?;
        Ok(())
    }
    pub async fn get_api_key(
        &self,
        api_key_hash: &str,
    ) -> Result<Option<ApiKeyRecord>, B2AStorageError> {
        let result = self
            .client
            .get_item()
            .table_name(&self.balances_table)
            .key(
                "address",
                AttributeValue::S(format!("__api_key:{}", api_key_hash)),
            )
            .send()
            .await
            .map_err(|e| B2AStorageError::Other(format!("DynamoDB get api key error: {:?}", e)))?;
        let Some(item) = result.item else {
            return Ok(None);
        };
        let Some(AttributeValue::S(serialized)) = item.get("api_key") else {
            return Ok(None);
        };
        serde_json::from_str(serialized)
            .map(Some)
            .map_err(|e| B2AStorageError::Other(format!("Invalid API key record: {}", e)))
    }
    pub async fn increment_api_key_usage(
        &self,
        api_key_hash: &str,
        amount: u64,
    ) -> Result<(), B2AStorageError> {
        let pk = format!("__api_key:{}", api_key_hash);
        let result = self
            .client
            .update_item()
            .table_name(&self.balances_table)
            .key("address", AttributeValue::S(pk.clone()))
            .update_expression("SET used_this_month = used_this_month + :amount")
            .condition_expression(
                "attribute_exists(address) AND active = :true AND used_this_month + :amount <= monthly_limit",
            )
            .expression_attribute_values(":amount", AttributeValue::N(amount.to_string()))
            .expression_attribute_values(":true", AttributeValue::Bool(true))
            .send()
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                let err_str = format!("{:?}", e);
                if err_str.contains("ConditionalCheckFailedException") {
                    // Could be invalid key, inactive key, or limit exceeded.
                    // For the sake of standard B2A semantics (which distinguishes these internally/externally in evaluation)
                    // we do a fast check to see which one it is.
                    let record = self.get_api_key(api_key_hash).await?;
                    if let Some(r) = record {
                        if !r.active {
                            return Err(B2AStorageError::Other("API key inactive".to_string()));
                        }
                        return Err(B2AStorageError::InsufficientFunds);
                    }
                    return Err(B2AStorageError::Other("API key not found".to_string()));
                }
                Err(B2AStorageError::Other(format!(
                    "DynamoDB api usage error: {}",
                    err_str
                )))
            }
        }
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
