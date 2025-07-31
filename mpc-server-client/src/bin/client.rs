use anyhow::Result;
use mpc_server_client::{MpcClient, utils::*, SessionId};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: {} <party_id> <server_url> [command] [args...]", args[0]);
        println!("Commands:");
        println!("  dkg <session_id> <threshold> <total_parties> - Start DKG");
        println!("  sign <session_id> <message> <party1> <party2> - Start signing");
        println!("  verify <session_id> <message> <r_hex> <s_hex> - Verify signature");
        return Ok(());
    }
    
    let party_id: u16 = args[1].parse()?;
    let server_url = args[2].clone();
    let command = args.get(3).map(|s| s.as_str()).unwrap_or("help");
    
    println!("=== MPC Client Starting ===");
    println!("Party ID: {}", party_id);
    println!("Server URL: {}", server_url);
    
    // Create client
    let storage_dir = format!("data/client_{}", party_id);
    let mut client = MpcClient::new(party_id, server_url, &storage_dir)?;
    
    match command {
        "dkg" => {
            if args.len() < 7 {
                println!("Usage: dkg <session_id> <threshold> <total_parties>");
                return Ok(());
            }
            
            let session_id_str = &args[4];
            let threshold: u16 = args[5].parse()?;
            let total_parties: u16 = args[6].parse()?;
            
            // Create session ID
            let session_id = if session_id_str == "new" {
                generate_session_id()
            } else {
                // Parse existing session ID
                let mut execution_id = [0u8; 32];
                // For simplicity, we'll use a fixed execution ID
                SessionId {
                    id: session_id_str.to_string(),
                    execution_id,
                }
            };
            
            println!("Starting DKG with session: {}", session_id.id);
            println!("Threshold: {}/{}", threshold, total_parties);
            
            client.start_dkg(session_id.clone(), threshold, total_parties).await?;
            
            // Print key share info
            if let Some(key_share) = client.get_key_share_info(&session_id.id)? {
                print_key_share_info(&key_share);
            }
        }
        
        "sign" => {
            if args.len() < 7 {
                println!("Usage: sign <session_id> <message> <party1> <party2>");
                return Ok(());
            }
            
            let session_id_str = &args[4];
            let message = &args[5];
            let party1: u16 = args[6].parse()?;
            let party2: u16 = args[7].parse()?;
            
            let session_id = SessionId {
                id: session_id_str.to_string(),
                execution_id: [0u8; 32], // Fixed for demo
            };
            
            println!("Starting signing with session: {}", session_id.id);
            println!("Message: {}", message);
            println!("Participating parties: [{}, {}]", party1, party2);
            
            let participating_parties = vec![party1, party2];
            let signature = client.start_signing(session_id.clone(), message, participating_parties).await?;
            
            print_signature_info(&signature, message);
            
            // Verify signature
            if let Some(key_share) = client.get_key_share_info(&session_id.id)? {
                let public_key = key_share.key_share.shared_public_key();
                client.verify_signature(&signature, message, &public_key)?;
                println!("✅ Signature verification successful!");
            }
        }
        
        "verify" => {
            if args.len() < 8 {
                println!("Usage: verify <session_id> <message> <r_hex> <s_hex>");
                return Ok(());
            }
            
            let session_id_str = &args[4];
            let message = &args[5];
            let r_hex = &args[6];
            let s_hex = &args[7];
            
            let session_id = SessionId {
                id: session_id_str.to_string(),
                execution_id: [0u8; 32],
            };
            
            let signature = parse_signature(r_hex, s_hex)?;
            
            if let Some(key_share) = client.get_key_share_info(&session_id.id)? {
                let public_key = key_share.key_share.shared_public_key();
                client.verify_signature(&signature, message, &public_key)?;
                println!("✅ Signature verification successful!");
            } else {
                println!("❌ Key share not found for session {}", session_id.id);
            }
        }
        
        "list-shares" => {
            if args.len() < 5 {
                println!("Usage: list-shares <session_id>");
                return Ok(());
            }
            
            let session_id_str = &args[4];
            let key_shares = client.list_key_shares(session_id_str)?;
            
            println!("=== Key Shares for Session {} ===", session_id_str);
            for key_share in key_shares {
                print_key_share_info(&key_share);
            }
        }
        
        "reconstruct" => {
            if args.len() < 5 {
                println!("Usage: reconstruct <session_id>");
                return Ok(());
            }
            
            let session_id_str = &args[4];
            let key_shares = client.list_key_shares(session_id_str)?;
            
            if key_shares.len() >= 2 {
                let private_key = reconstruct_private_key(&key_shares)?;
                let private_key_hex = private_key_to_hex(&private_key);
                
                println!("=== Reconstructed Private Key ===");
                println!("Private Key (hex): {}", private_key_hex);
                
                // Verify against public key
                if let Some(key_share) = key_shares.first() {
                    let public_key = key_share.key_share.shared_public_key();
                    let is_valid = verify_private_key(&private_key, &public_key)?;
                    println!("Verification: {}", if is_valid { "✅ Valid" } else { "❌ Invalid" });
                }
            } else {
                println!("❌ Need at least 2 key shares to reconstruct private key");
            }
        }
        
        _ => {
            println!("Unknown command: {}", command);
            println!("Available commands: dkg, sign, verify, list-shares, reconstruct");
        }
    }
    
    Ok(())
} 