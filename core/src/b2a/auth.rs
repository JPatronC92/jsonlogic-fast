use siwe::{Message, VerificationOpts};
use std::str::FromStr;

/// Verifica una firma SIWE de forma estricta.
/// 
/// - Valida la firma criptográfica.
/// - (Opcional) Valida domain y uri esperados.
/// - Devuelve el address y el Message completo para extraer nonce sin re-parsing.
pub async fn verify_siwe(
    message_str: &str,
    signature_hex: &str,
    expected_domain: Option<&str>,
    expected_uri: Option<&str>,
) -> Result<(String, Message), String> {
    let message =
        Message::from_str(message_str).map_err(|e| format!("Invalid SIWE message: {}", e))?;

    let sig_bytes = hex::decode(signature_hex.trim_start_matches("0x"))
        .map_err(|e| format!("Invalid signature hex: {}", e))?;

    let signature: [u8; 65] = sig_bytes
        .try_into()
        .map_err(|_| "Signature must be 65 bytes long".to_string())?;

    let mut opts = VerificationOpts::default();

    if let Some(domain) = expected_domain {
        opts.domain = Some(domain.parse().map_err(|_| "Invalid domain".to_string())?);
    }

    // Validación estricta de URI (si se provee) para cumplir con recomendaciones de S1 del roadmap
    if let Some(uri) = expected_uri {
        // Verificación post-parse: el URI del mensaje debe coincidir (case-insensitive en host/path básico)
        let msg_uri = message.uri.to_string();
        if !msg_uri.eq_ignore_ascii_case(uri) && !msg_uri.starts_with(uri) {
            return Err(format!("URI mismatch: expected {}, got {}", uri, msg_uri));
        }
    }

    // Validación criptográfica + domain (si se configuró)
    match message.verify(&signature, &opts).await {
        Ok(_) => {
            let address = format!("0x{}", hex::encode(message.address));
            Ok((address, message))
        }
        Err(e) => Err(format!("Signature verification failed: {}", e)),
    }
}
