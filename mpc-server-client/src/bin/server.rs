use anyhow::Result;
use mpc_server_client::MpcServer;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("=== MPC Server Starting ===");
    println!("Server will handle DKG and signing protocols");
    println!("Supporting 2/3 threshold for secp256k1");
    
    // Create and start server
    let server = MpcServer::new("data/server", "127.0.0.1:3000")?;
    
    println!("Server starting on http://127.0.0.1:3000");
    println!("API endpoints:");
    println!("  POST /api/mpc/dkg - Start DKG protocol");
    println!("  POST /api/mpc/sign - Start signing protocol");
    println!("  GET  /api/mpc/sessions - List all sessions");
    println!("  GET  /api/mpc/key-shares/:session_id - Get key shares");
    println!("  POST /api/mpc/message - Receive protocol message");
    println!("  GET  /api/mpc/messages/:session_id/:round - Get round messages");
    
    server.start().await?;
    
    Ok(())
} 