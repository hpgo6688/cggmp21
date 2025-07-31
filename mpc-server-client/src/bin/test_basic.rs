use anyhow::Result;
use mpc_server_client::{MpcClient, utils::*};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Basic MPC Test ===");
    
    // Test session ID generation
    let session_id = generate_session_id();
    println!("Generated session ID: {}", session_id.id);
    
    // Test party info creation
    let parties = create_test_parties();
    println!("Created {} test parties", parties.len());
    
    // Test signature parsing and formatting
    let test_r = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
    let test_s = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
    
    match parse_signature(test_r, test_s) {
        Ok(signature) => {
            let formatted = format_signature(&signature);
            println!("Parsed and formatted signature: {}", formatted);
        }
        Err(e) => {
            println!("Signature parsing test failed: {}", e);
        }
    }
    
    // Test private key utilities
    let test_private_key = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32];
    let hex_key = private_key_to_hex(&test_private_key);
    println!("Private key to hex: {}", hex_key);
    
    match hex_to_private_key(&hex_key) {
        Ok(decoded) => {
            if decoded == test_private_key {
                println!("✅ Private key encoding/decoding test passed");
            } else {
                println!("❌ Private key encoding/decoding test failed");
            }
        }
        Err(e) => {
            println!("❌ Private key decoding failed: {}", e);
        }
    }
    
    println!("=== Basic test completed ===");
    Ok(())
} 