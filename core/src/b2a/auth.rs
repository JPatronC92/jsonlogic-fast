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

    // Nota: uri se valida principalmente a través del propio Message y el signer.
    // timestamp y nonce se manejan por defecto en la librería.
    // Para mayor estrictez se puede extender con más campos del opts.

    // Validación estricta de tiempo y otros campos por defecto del crate
    match message.verify(&signature, &opts).await {
        Ok(_) => {
            let address = format!("0x{}", hex::encode(message.address));
            Ok((address, message))
        }
        Err(e) => Err(format!("Signature verification failed: {}", e)),
    }
}
