use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use aws_sdk_dynamodb::{Client, types::AttributeValue};
use std::time::{SystemTime, UNIX_EPOCH};

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

    pub async fn get_balance(&self, address: &str) -> Result<f64, String> {
        match self {
            StorageBackend::Memory(m) => m.get_balance(address).await,
            StorageBackend::Dynamo(d) => d.get_balance(address).await,
        }
    }

    pub async fn deduct_balance(&self, address: &str, cost: f64) -> Result<(), String> {
        match self {
            StorageBackend::Memory(m) => m.deduct_balance(address, cost).await,
            StorageBackend::Dynamo(d) => d.deduct_balance(address, cost).await,
        }
    }

    pub async fn add_balance(&self, address: &str, amount: f64) -> Result<(), String> {
        match self {
            StorageBackend::Memory(m) => m.add_balance(address, amount).await,
            StorageBackend::Dynamo(d) => d.add_balance(address, amount).await,
        }
    }

    pub async fn get_all_balances(&self) -> Result<Vec<(String, f64)>, String> {
        match self {
            StorageBackend::Memory(m) => m.get_all_balances().await,
            StorageBackend::Dynamo(d) => d.get_all_balances().await,
        }
    }
}

#[derive(Clone)]
pub struct MemoryStorage {
    balances: Arc<Mutex<HashMap<String, f64>>>,
    nonces: Arc<Mutex<HashSet<String>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            balances: Arc::new(Mutex::new(HashMap::new())),
            nonces: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}

impl MemoryStorage {
    pub async fn check_and_record_nonce(&self, address: &str, nonce: &str) -> Result<bool, String> {
        let mut nonces = self.nonces.lock().await;
        let pk = format!("{}#{}", address, nonce);
        if nonces.contains(&pk) {
            Ok(false)
        } else {
            nonces.insert(pk);
            Ok(true)
        }
    }

    pub async fn get_balance(&self, address: &str) -> Result<f64, String> {
        let balances = self.balances.lock().await;
        Ok(*balances.get(address).unwrap_or(&10.0)) // Default seed
    }

    pub async fn deduct_balance(&self, address: &str, cost: f64) -> Result<(), String> {
        let mut balances = self.balances.lock().await;
        let current = *balances.get(address).unwrap_or(&10.0);
        if current < cost {
            return Err("Insufficient funds".to_string());
        }
        balances.insert(address.to_string(), current - cost);
        Ok(())
    }

    pub async fn add_balance(&self, address: &str, amount: f64) -> Result<(), String> {
        let mut balances = self.balances.lock().await;
        let current = *balances.get(address).unwrap_or(&10.0);
        balances.insert(address.to_string(), current + amount);
        Ok(())
    }

    pub async fn get_all_balances(&self) -> Result<Vec<(String, f64)>, String> {
        let cache = self.balances.lock().await;
        let mut results = Vec::new();
        for (k, v) in cache.iter() {
            results.push((k.clone(), *v));
        }
        Ok(results)
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

    pub async fn get_balance(&self, address: &str) -> Result<f64, String> {
        let result = self.client.get_item()
            .table_name(&self.balances_table)
            .key("address", AttributeValue::S(address.to_string()))
            .send()
            .await
            .map_err(|e| format!("DynamoDB get error: {:?}", e))?;

        if let Some(item) = result.item {
            if let Some(AttributeValue::N(bal_str)) = item.get("balance") {
                return bal_str.parse::<f64>().map_err(|_| "Invalid balance format".to_string());
            }
        }
        
        // Default seed balance for testing
        Ok(10.0)
    }

    pub async fn deduct_balance(&self, address: &str, cost: f64) -> Result<(), String> {
        let current = self.get_balance(address).await?;
        if current < cost {
            return Err("Insufficient funds".to_string());
        }

        let new_balance = current - cost;

        self.client.put_item()
            .table_name(&self.balances_table)
            .item("address", AttributeValue::S(address.to_string()))
            .item("balance", AttributeValue::N(new_balance.to_string()))
            .send()
            .await
            .map_err(|e| format!("DynamoDB put error: {:?}", e))?;

        Ok(())
    }

    pub async fn add_balance(&self, address: &str, amount: f64) -> Result<(), String> {
        let current = self.get_balance(address).await.unwrap_or(0.0);
        let new_balance = current + amount;

        self.client.put_item()
            .table_name(&self.balances_table)
            .item("address", AttributeValue::S(address.to_string()))
            .item("balance", AttributeValue::N(new_balance.to_string()))
            .send()
            .await
            .map_err(|e| format!("DynamoDB put error: {:?}", e))?;

        Ok(())
    }

    pub async fn get_all_balances(&self) -> Result<Vec<(String, f64)>, String> {
        let mut scan = self.client.scan().table_name(&self.balances_table).into_paginator().send();
        let mut results = Vec::new();
        
        while let Some(result) = scan.next().await {
            match result {
                Ok(output) => {
                    if let Some(items) = output.items {
                        for item in items {
                            if let (Some(id_val), Some(bal_val)) = (item.get("address"), item.get("balance")) {
                                if let (Ok(id), Ok(bal_str)) = (id_val.as_s(), bal_val.as_n()) {
                                    if let Ok(bal) = bal_str.parse::<f64>() {
                                        results.push((id.clone(), bal));
                                    }
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
}
