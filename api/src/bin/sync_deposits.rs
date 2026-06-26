use alloy::providers::Provider;
use alloy::rpc::types::eth::Filter;
use alloy::sol_types::SolEvent;
use alloy_primitives::U256;
use api::blockchain::{retry_with_backoff, B2AStaking, BlockchainConfig};
use api::storage::{DynamoStorage, MemoryStorage, StorageBackend};
use serde_json::json;
use std::env;
use std::sync::Arc;

fn log(level: &str, msg: &str, extra: Option<serde_json::Value>) {
    let mut log_obj = json!({
        "level": level,
        "msg": msg
    });
    if let Some(e) = extra {
        if let Some(obj) = e.as_object() {
            for (k, v) in obj {
                log_obj[k] = v.clone();
            }
        } else {
            log_obj["data"] = e;
        }
    }
    println!("{}", log_obj);
}

fn log_info(msg: &str, data: Option<serde_json::Value>) {
    log("INFO", msg, data);
}

fn log_error(msg: &str, data: Option<serde_json::Value>) {
    log("ERROR", msg, data);
}

/// Sync deposits usando polling HTTP (diseñado para ejecución programada en Lambda).
///
/// - Recupera el último bloque procesado desde storage.
/// - Procesa un rango acotado de bloques.
/// - Persiste el último bloque procesado para reanudar sin pérdida.
/// - Idempotente y seguro para reintentos.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let balances_table = env::var("BALANCES_TABLE").unwrap_or_else(|_| "B2A_Balances".to_string());
    let nonces_table = env::var("NONCES_TABLE").unwrap_or_else(|_| "B2A_Nonces".to_string());
    let use_dynamo = env::var("USE_DYNAMODB").unwrap_or_else(|_| "false".to_string()) == "true";

    let storage: Arc<StorageBackend> = if use_dynamo {
        log_info("storage_init", Some(json!({ "backend": "dynamodb" })));
        Arc::new(StorageBackend::Dynamo(
            DynamoStorage::new(&balances_table, &nonces_table).await,
        ))
    } else {
        log_info("storage_init", Some(json!({ "backend": "memory" })));
        Arc::new(StorageBackend::Memory(MemoryStorage::new()))
    };

    let config = BlockchainConfig::from_env();
    let provider = config.read_provider();

    log_info(
        "sync_start",
        Some(json!({ "contract": config.contract_address.to_string() })),
    );

    // Recuperar último bloque procesado (con fallback seguro)
    let mut last_block: u64 = match storage.get_last_sync_block().await {
        Ok(b) => b,
        Err(e) => {
            log_error("get_last_block_failed", Some(json!({ "error": e })));
            0
        }
    };

    // Primera ejecución: backfill reciente (configurable)
    if last_block == 0 {
        match retry_with_backoff(|| async { provider.get_block_number().await }, 3, 500).await {
            Ok(latest) => {
                last_block = latest.saturating_sub(200);
                log_info(
                    "first_run_backfill",
                    Some(json!({ "start_block": last_block })),
                );
            }
            Err(e) => {
                log_error(
                    "get_latest_block_failed",
                    Some(json!({ "error": e.to_string() })),
                );
                return Ok(());
            }
        }
    }

    let from_block = last_block + 1;

    let latest =
        match retry_with_backoff(|| async { provider.get_block_number().await }, 3, 500).await {
            Ok(n) => n,
            Err(e) => {
                log_error(
                    "get_latest_block_failed",
                    Some(json!({ "error": e.to_string() })),
                );
                return Ok(());
            }
        };

    // Límite de bloques por ejecución (ajustable vía env para controlar costos/latency)
    let max_blocks: u64 = env::var("SYNC_MAX_BLOCKS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2000);

    let to_block = std::cmp::min(latest, from_block + max_blocks.saturating_sub(1));

    if to_block < from_block {
        log_info(
            "no_new_blocks",
            Some(json!({ "last_processed": last_block })),
        );
        return Ok(());
    }

    log_info(
        "processing_range",
        Some(json!({ "from": from_block, "to": to_block })),
    );

    let filter = Filter::new()
        .address(config.contract_address)
        .from_block(from_block)
        .to_block(to_block)
        .event_signature(B2AStaking::Deposited::SIGNATURE_HASH);

    let logs = match retry_with_backoff(|| async { provider.get_logs(&filter).await }, 3, 500).await
    {
        Ok(l) => l,
        Err(e) => {
            log_error(
                "get_logs_failed",
                Some(json!({ "error": e.to_string(), "from": from_block, "to": to_block })),
            );
            return Ok(()); // No avanzar cursor para reintento
        }
    };

    let mut highest_processed = last_block;
    let mut processed = 0u64;
    let mut failures = 0u64;

    for log_entry in logs {
        if let Ok(decoded) = log_entry.log_decode::<B2AStaking::Deposited>() {
            let event = decoded.inner.data;
            let user = event.user.to_string();
            let amount: U256 = event.amount;
            let block_num = log_entry.block_number.unwrap_or_default();

            log_info(
                "deposit_detected",
                Some(json!({
                    "user": user,
                    "amount": amount.to_string(),
                    "block": block_num
                })),
            );

            if let Err(e) = storage.add_balance(&user, amount).await {
                log_error(
                    "add_balance_failed",
                    Some(json!({
                        "user": user,
                        "amount": amount.to_string(),
                        "block": block_num,
                        "error": e
                    })),
                );
                failures += 1;
                // No avanzamos para este evento; reintentar en próxima ejecución
                continue;
            }

            processed += 1;
            if block_num > highest_processed {
                highest_processed = block_num;
            }
        }
    }

    // Persistir progreso solo si hubo avances
    if highest_processed > last_block {
        if let Err(e) = storage.set_last_sync_block(highest_processed).await {
            log_error(
                "persist_last_block_failed",
                Some(json!({ "error": e, "block": highest_processed })),
            );
        } else {
            log_info(
                "last_block_saved",
                Some(json!({ "block": highest_processed })),
            );
        }
    }

    log_info(
        "sync_batch_complete",
        Some(json!({
            "processed": processed,
            "failures": failures,
            "up_to_block": highest_processed
        })),
    );

    Ok(())
}
