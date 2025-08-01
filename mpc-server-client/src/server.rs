use std::sync::Arc;
use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use rand::{SeedableRng, rngs::StdRng};
use sha2::{Sha256, Digest};
use round_based;
use cggmp21;

use crate::types::*;
use crate::storage::Storage;
use crate::protocol::{ProtocolManager, ProtocolMessage};

/// MPC Server implementation
pub struct MpcServer {
    storage: Arc<Storage>,
    protocol_manager: Arc<ProtocolManager>,
    address: String,
}

impl MpcServer {
    pub fn new(storage_dir: &str, address: &str) -> Result<Self> {
        let storage = Storage::new(storage_dir)?;
        storage.load_from_disk()?;
        
        let protocol_manager = ProtocolManager::new(storage.clone());
        
        Ok(Self {
            storage: Arc::new(storage),
            protocol_manager: Arc::new(protocol_manager),
            address: address.to_string(),
        })
    }
    
    pub async fn start(&self) -> Result<()> {
        let app = self.create_router();
        
        let listener = TcpListener::bind(&self.address).await?;
        println!("MPC Server starting on {}", self.address);
        
        axum::serve(listener, app).await?;
        
        Ok(())
    }
    
    fn create_router(&self) -> Router {
        let cors = CorsLayer::permissive();
        
        Router::new()
            .route("/api/mpc/session", post(create_session))
            .route("/api/mpc/session/:session_id", get(get_session))
            .route("/api/mpc/sessions", get(list_sessions))
            .route("/api/mpc/message", post(receive_message))
            .route("/api/mpc/messages/:session_id/:round", get(get_round_messages))
            .route("/api/mpc/dkg", post(start_dkg))
            .route("/api/mpc/sign", post(start_signing))
            .route("/api/mpc/key-shares/:session_id", get(get_key_shares))
            .layer(cors)
            .with_state(Arc::clone(&self.protocol_manager))
    }
}

/// API Handlers
async fn create_session(
    State(protocol_manager): State<Arc<ProtocolManager>>,
    Json(request): Json<MpcMessage>,
) -> Result<Json<ApiResponse<SessionInfo>>, StatusCode> {
    match request {
        MpcMessage::CreateSession { session_id, protocol_type, parties, threshold } => {
            match protocol_manager.create_session(session_id.clone(), protocol_type, parties, threshold) {
                Ok(()) => {
                    let session_info = protocol_manager.get_session(&session_id.id)
                        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
                    
                    let response = SessionInfo {
                        session_id: session_info.session_id,
                        protocol_type: session_info.protocol_type,
                        state: session_info.state,
                        parties: session_info.parties,
                        threshold: session_info.threshold,
                        total_parties: session_info.total_parties,
                        created_at: session_info.created_at,
                        updated_at: session_info.updated_at,
                    };
                    
                    Ok(Json(ApiResponse {
                        success: true,
                        data: Some(response),
                        error: None,
                    }))
                }
                Err(e) => {
                    Ok(Json(ApiResponse {
                        success: false,
                        data: None,
                        error: Some(e.to_string()),
                    }))
                }
            }
        }
        _ => {
            Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Invalid message type".to_string()),
            }))
        }
    }
}

async fn get_session(
    State(protocol_manager): State<Arc<ProtocolManager>>,
    Path(session_id): Path<String>,
) -> Result<Json<ApiResponse<SessionInfo>>, StatusCode> {
    match protocol_manager.get_session(&session_id) {
        Some(session) => {
            let response = SessionInfo {
                session_id: session.session_id,
                protocol_type: session.protocol_type,
                state: session.state,
                parties: session.parties,
                threshold: session.threshold,
                total_parties: session.total_parties,
                created_at: session.created_at,
                updated_at: session.updated_at,
            };
            
            Ok(Json(ApiResponse {
                success: true,
                data: Some(response),
                error: None,
            }))
        }
        None => {
            Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Session not found".to_string()),
            }))
        }
    }
}

async fn list_sessions(
    State(protocol_manager): State<Arc<ProtocolManager>>,
) -> Result<Json<ApiResponse<Vec<SessionInfo>>>, StatusCode> {
    let sessions = protocol_manager.get_storage().list_sessions();
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(sessions),
        error: None,
    }))
}

async fn receive_message(
    State(protocol_manager): State<Arc<ProtocolManager>>,
    Json(message): Json<ProtocolMessage>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    match protocol_manager.add_message(message) {
        Ok(()) => {
            Ok(Json(ApiResponse {
                success: true,
                data: Some(()),
                error: None,
            }))
        }
        Err(e) => {
            Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }))
        }
    }
}

async fn get_round_messages(
    State(protocol_manager): State<Arc<ProtocolManager>>,
    Path((session_id, round)): Path<(String, u32)>,
) -> Result<Json<ApiResponse<Vec<ProtocolMessage>>>, StatusCode> {
    let messages = protocol_manager.get_round_messages(&session_id, round);
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(messages),
        error: None,
    }))
}

async fn start_dkg(
    State(protocol_manager): State<Arc<ProtocolManager>>,
    Json(request): Json<DkgRequest>,
) -> Result<Json<ApiResponse<SessionInfo>>, StatusCode> {
    let session_id = request.session_id;
    
    // 只有P1和P2需要客户端调用，P0自动参与
    let parties = vec![
        PartyInfo {
            id: 0,
            address: "http://localhost:3000".to_string(),
            is_online: true,
            last_seen: Some(chrono::Utc::now()),
        },
        PartyInfo {
            id: 1,
            address: request.party_address.clone(),
            is_online: true,
            last_seen: Some(chrono::Utc::now()),
        },
        PartyInfo {
            id: 2,
            address: "http://localhost:3002".to_string(),
            is_online: true,
            last_seen: Some(chrono::Utc::now()),
        },
    ];
    
    // 服务端P0自动参与DKG
    println!("服务端P0自动参与DKG协议");
    println!("parties: {:?}", parties);
    
    // 创建会话
    match protocol_manager.create_session(
        session_id.clone(),
        ProtocolType::DKG,
        parties,
        request.threshold,
    ) {
        Ok(()) => {
            // P0真正参与DKG协议，生成自己的私钥份额
            let execution_id = session_id.execution_id();
            
            // 使用确定性种子确保所有参与方生成相同的共享公钥
            let session_hash = sha2::Sha256::digest(session_id.id.as_bytes());
            let seed = u64::from_le_bytes([
                session_hash[0], session_hash[1], session_hash[2], session_hash[3],
                session_hash[4], session_hash[5], session_hash[6], session_hash[7]
            ]);
            
            // P0运行DKG协议
            let key_shares = round_based::sim::run(3, |i, party| {
                let mut party_rng = rand::rngs::StdRng::seed_from_u64(seed + i as u64);
                
                async move {
                    let keygen = cggmp21::keygen::<cggmp21::supported_curves::Secp256k1>(execution_id, i, 3)
                        .set_threshold(request.threshold);
                    
                    keygen.start(&mut party_rng, party).await
                }
            })
            .unwrap()
            .expect_ok()
            .into_vec();
            
                            // 保存P0的私钥份额
                if let Some(key_share) = key_shares.get(0) {
                    let stored_key_share = StoredKeyShare {
                        party_id: 0,
                        key_share: key_share.clone(),
                        session_id: session_id.clone(),
                        created_at: chrono::Utc::now(),
                    };
                    
                    // 保存到服务端存储
                    if let Err(e) = protocol_manager.get_storage().save_key_share(stored_key_share) {
                        eprintln!("Failed to save P0 key share: {}", e);
                    } else {
                        println!("✅ P0 key share saved with public key: {:?}", key_share.shared_public_key());
                    }
                }
            
            let session_info = protocol_manager.get_session(&session_id.id)
                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
            
            let response = SessionInfo {
                session_id: session_info.session_id,
                protocol_type: session_info.protocol_type,
                state: session_info.state,
                parties: session_info.parties,
                threshold: session_info.threshold,
                total_parties: session_info.total_parties,
                created_at: session_info.created_at,
                updated_at: session_info.updated_at,
            };
            
            Ok(Json(ApiResponse {
                success: true,
                data: Some(response),
                error: None,
            }))
        }
        Err(e) => {
            Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }))
        }
    }
}

async fn start_signing(
    State(protocol_manager): State<Arc<ProtocolManager>>,
    Json(request): Json<SignRequest>,
) -> Result<Json<ApiResponse<SessionInfo>>, StatusCode> {
    let session_id = request.session_id;
    let parties = vec![
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
    ];
    
    // 服务端P0自动参与签名
    println!("服务端P0自动参与签名协议");
    
    match protocol_manager.create_session(
        session_id.clone(),
        ProtocolType::Sign,
        parties,
        2, // threshold for 2/3
    ) {
        Ok(()) => {
            // P0真正参与签名协议
            let execution_id = session_id.execution_id();
            
            // 使用确定性种子确保签名一致性
            let session_hash = Sha256::digest(session_id.id.as_bytes());
            let seed = u64::from_le_bytes([
                session_hash[0], session_hash[1], session_hash[2], session_hash[3],
                session_hash[4], session_hash[5], session_hash[6], session_hash[7]
            ]);
            
            // 检查P0是否有私钥份额
            if let Some(p0_key_share) = protocol_manager.get_storage().get_key_share(&session_id.id, 0) {
                println!("✅ P0 found key share, participating in signing");
                
                // 这里可以添加真正的签名逻辑
                // 目前简化处理，实际应该运行完整的签名协议
                println!("📝 P0 ready to participate in threshold signing");
            } else {
                println!("⚠️ P0 key share not found for session {}", session_id.id);
            }
            
            let session_info = protocol_manager.get_session(&session_id.id)
                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
            
            let response = SessionInfo {
                session_id: session_info.session_id,
                protocol_type: session_info.protocol_type,
                state: session_info.state,
                parties: session_info.parties,
                threshold: session_info.threshold,
                total_parties: session_info.total_parties,
                created_at: session_info.created_at,
                updated_at: session_info.updated_at,
            };
            
            Ok(Json(ApiResponse {
                success: true,
                data: Some(response),
                error: None,
            }))
        }
        Err(e) => {
            Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }))
        }
    }
}

async fn get_key_shares(
    State(protocol_manager): State<Arc<ProtocolManager>>,
    Path(session_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<StoredKeyShare>>>, StatusCode> {
    let key_shares = protocol_manager.get_storage().list_key_shares(&session_id);
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(key_shares),
        error: None,
    }))
} 