use anyhow::Result;
use cggmp21::{
    supported_curves::Secp256k1,
    key_share::reconstruct_secret_key,
    signing::Signature,
};
use generic_ec::Point;
use hex;

use crate::types::*;

/// Utility functions for MPC operations

/// Reconstruct the complete private key from key shares
pub fn reconstruct_private_key(key_shares: &[StoredKeyShare]) -> Result<Vec<u8>> {
    let shares: Vec<_> = key_shares.iter().map(|ks| &ks.key_share).collect();
    
    let secret_key = reconstruct_secret_key(&shares)
        .map_err(|e| anyhow::anyhow!("Failed to reconstruct secret key: {:?}", e))?;
    
    let scalar_value = secret_key.as_ref();
    let private_key_bytes = scalar_value.to_be_bytes();
    
    Ok(private_key_bytes.to_vec())
}

/// Convert private key bytes to hex string
pub fn private_key_to_hex(private_key: &[u8]) -> String {
    hex::encode(private_key)
}

/// Convert hex string to private key bytes
pub fn hex_to_private_key(hex_str: &str) -> Result<Vec<u8>> {
    hex::decode(hex_str)
        .map_err(|e| anyhow::anyhow!("Invalid hex string: {}", e))
}

/// Verify that reconstructed private key matches the public key
pub fn verify_private_key(private_key: &[u8], public_key: &Point<Secp256k1>) -> Result<bool> {
    use generic_ec::Scalar;
    
    let scalar = Scalar::from_be_bytes_mod_order(private_key);
    let reconstructed_public_key = Point::generator() * &scalar;
    
    Ok(reconstructed_public_key == *public_key)
}

/// Format signature for display
pub fn format_signature(signature: &Signature<Secp256k1>) -> String {
    let r_hex = hex::encode(signature.r.as_ref().to_be_bytes());
    let s_hex = hex::encode(signature.s.as_ref().to_be_bytes());
    format!("r: {}, s: {}", r_hex, s_hex)
}

/// Parse signature from hex strings
pub fn parse_signature(r_hex: &str, s_hex: &str) -> Result<Signature<Secp256k1>> {
    use generic_ec::{Scalar, NonZero};
    
    let r_bytes = hex::decode(r_hex)
        .map_err(|e| anyhow::anyhow!("Invalid r hex: {}", e))?;
    let s_bytes = hex::decode(s_hex)
        .map_err(|e| anyhow::anyhow!("Invalid s hex: {}", e))?;
    
    let r = Scalar::from_be_bytes_mod_order(&r_bytes);
    let s = Scalar::from_be_bytes_mod_order(&s_bytes);
    
    let r = NonZero::from_scalar(r)
        .ok_or_else(|| anyhow::anyhow!("r is zero"))?;
    let s = NonZero::from_scalar(s)
        .ok_or_else(|| anyhow::anyhow!("s is zero"))?;
    
    Ok(Signature { r, s })
}

/// Generate a random session ID
pub fn generate_session_id() -> SessionId {
    SessionId::new()
}

/// Create party info for testing
pub fn create_test_parties() -> Vec<PartyInfo> {
    vec![
        PartyInfo {
            id: 0,
            address: "http://localhost:3000".to_string(),
            is_online: true,
            last_seen: Some(chrono::Utc::now()),
        },
        PartyInfo {
            id: 1,
            address: "http://localhost:3001".to_string(),
            is_online: true,
            last_seen: Some(chrono::Utc::now()),
        },
        PartyInfo {
            id: 2,
            address: "http://localhost:3002".to_string(),
            is_online: true,
            last_seen: Some(chrono::Utc::now()),
        },
    ]
}

/// Print key share information
pub fn print_key_share_info(key_share: &StoredKeyShare) {
    println!("=== Key Share Information ===");
    println!("Party ID: {}", key_share.party_id);
    println!("Session ID: {}", key_share.session_id.id);
    println!("Shared Public Key: {:?}", key_share.key_share.shared_public_key());
    println!("Created At: {}", key_share.created_at);
    println!("=============================");
}

/// Print signature information
pub fn print_signature_info(signature: &Signature<Secp256k1>, message: &str) {
    println!("=== Signature Information ===");
    println!("Message: {}", message);
    println!("Signature: {}", format_signature(signature));
    println!("============================");
}

/// Print session information
pub fn print_session_info(session: &SessionInfo) {
    println!("=== Session Information ===");
    println!("Session ID: {}", session.session_id.id);
    println!("Protocol Type: {:?}", session.protocol_type);
    println!("State: {:?}", session.state);
    println!("Threshold: {}/{}", session.threshold, session.total_parties);
    println!("Parties:");
    for party in &session.parties {
        println!("  - Party {}: {} ({})", 
            party.id, party.address, 
            if party.is_online { "online" } else { "offline" });
    }
    println!("Created At: {}", session.created_at);
    println!("Updated At: {}", session.updated_at);
    println!("===========================");
} 