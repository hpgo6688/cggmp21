use anyhow::Result;
use mpc_server_client::{MpcClient, utils::*};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Simple MPC Demo ===");
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <party_id>", args[0]);
        println!("This demo will:");
        println!("1. Run DKG with 3 parties (2/3 threshold)");
        println!("2. Test signing with different party combinations");
        println!("3. Verify key consistency");
        return Ok(());
    }
    
    let party_id: u16 = args[1].parse()?;
    let server_url = "http://127.0.0.1:3000".to_string();
    
    println!("Party ID: {}", party_id);
    println!("Server URL: {}", server_url);
    
    // Create client
    let storage_dir = format!("data/client_{}", party_id);
    let mut client = MpcClient::new(party_id, server_url, &storage_dir)?;
    
    // Generate session ID
    let session_id = generate_session_id();
    println!("Session ID: {}", session_id.id);
    
    // Step 1: Run DKG
    println!("\n=== Step 1: Running DKG ===");
    client.start_dkg(session_id.clone(), 2, 3).await?;
    
    // Step 2: Test signing
    println!("\n=== Step 2: Testing Signing ===");
    let message = "hello world";
    let participating_parties = vec![0, 1]; // P0 and P1
    
    let signature = client.start_signing(session_id.clone(), message, participating_parties).await?;
    println!("Generated signature: {}", format_signature(&signature));
    
    // Step 3: Verify signature
    println!("\n=== Step 3: Verifying Signature ===");
    if let Some(key_share) = client.get_key_share_info(&session_id.id)? {
        let public_key = key_share.key_share.shared_public_key();
        client.verify_signature(&signature, message, &public_key)?;
        println!("✅ Signature verification successful!");
    }
    
    // Step 4: List key shares
    println!("\n=== Step 4: Key Share Information ===");
    let key_shares = client.list_key_shares(&session_id.id)?;
    for key_share in key_shares {
        print_key_share_info(&key_share);
    }
    
    // Step 5: Test private key reconstruction
    println!("\n=== Step 5: Private Key Reconstruction ===");
    if key_shares.len() >= 2 {
        match reconstruct_private_key(&key_shares) {
            Ok(private_key) => {
                let private_key_hex = private_key_to_hex(&private_key);
                println!("✅ Reconstructed private key: {}", private_key_hex);
                
                // Verify against public key
                if let Some(key_share) = key_shares.first() {
                    let public_key = key_share.key_share.shared_public_key();
                    let is_valid = verify_private_key(&private_key, &public_key)?;
                    println!("Private key verification: {}", if is_valid { "✅ Valid" } else { "❌ Invalid" });
                }
            }
            Err(e) => {
                println!("❌ Failed to reconstruct private key: {}", e);
            }
        }
    } else {
        println!("❌ Need at least 2 key shares to reconstruct private key");
    }
    
    println!("\n=== Demo completed successfully! ===");
    Ok(())
} 