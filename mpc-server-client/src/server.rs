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
    
    match protocol_manager.create_session(
        session_id.clone(),
        ProtocolType::DKG,
        parties,
        request.threshold,
    ) {
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
    
    match protocol_manager.create_session(
        session_id.clone(),
        ProtocolType::Sign,
        parties,
        2, // threshold for 2/3
    ) {
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