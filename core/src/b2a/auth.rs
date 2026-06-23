use siwe::{Message, VerificationOpts};
use std::str::FromStr;

pub async fn verify_siwe(message_str: &str, signature_hex: &str) -> Result<String, String> {
    let message =
        Message::from_str(message_str).map_err(|e| format!("Invalid SIWE message: {}", e))?;

    let sig_bytes = hex::decode(signature_hex.trim_start_matches("0x"))
        .map_err(|e| format!("Invalid signature hex: {}", e))?;

    let signature: [u8; 65] = sig_bytes
        .try_into()
        .map_err(|_| "Signature must be 65 bytes long".to_string())?;

    match message
        .verify(&signature, &VerificationOpts::default())
        .await
    {
        Ok(_) => {
            let address = format!("0x{}", hex::encode(message.address));
            Ok(address)
        }
        Err(e) => Err(format!("Signature verification failed: {}", e)),
    }
}
