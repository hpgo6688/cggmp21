use serde::{Deserialize, Serialize};
use cggmp21::{
    ExecutionId, supported_curves::Secp256k1, 
    key_share::{DirtyIncompleteKeyShare, Valid},
    signing::{Signature, DataToSign, Presignature, PartialSignature}
};
use generic_ec::Point;
use std::collections::HashMap;
use rand::{Rng, RngCore};

/// Party identifier
pub type PartyId = u16;

/// Session identifier for MPC operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionId {
    pub id: String,
    pub execution_id: [u8; 32],
}

/// MPC Protocol types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolType {
    DKG,    // Distributed Key Generation
    Sign,   // Threshold Signing
}

/// MPC Protocol state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolState {
    Initialized,
    InProgress,
    Completed,
    Failed(String),
}

/// Party information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyInfo {
    pub id: PartyId,
    pub address: String,
    pub is_online: bool,
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: SessionId,
    pub protocol_type: ProtocolType,
    pub state: ProtocolState,
    pub parties: Vec<PartyInfo>,
    pub threshold: u16,
    pub total_parties: u16,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Key share information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyShareInfo {
    pub party_id: PartyId,
    pub shared_public_key: String,
    pub threshold: u16,
    pub total_parties: u16,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Message types for HTTP communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MpcMessage {
    // Session management
    CreateSession {
        session_id: SessionId,
        protocol_type: ProtocolType,
        parties: Vec<PartyInfo>,
        threshold: u16,
    },
    JoinSession {
        session_id: SessionId,
        party_id: PartyId,
        party_address: String,
    },
    GetSessionInfo {
        session_id: SessionId,
    },
    
    // DKG messages
    DkgMessage {
        session_id: SessionId,
        party_id: PartyId,
        round: u32,
        message: Vec<u8>,
    },
    
    // Signing messages
    SignMessage {
        session_id: SessionId,
        party_id: PartyId,
        round: u32,
        message: Vec<u8>,
    },
    
    // Results
    DkgResult {
        session_id: SessionId,
        key_shares: HashMap<PartyId, KeyShareInfo>,
    },
    SignResult {
        session_id: SessionId,
        signature: Signature<Secp256k1>,
        message: String,
    },
    
    // Error
    Error {
        session_id: SessionId,
        error: String,
    },
}

/// HTTP API Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

/// DKG Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DkgRequest {
    pub session_id: SessionId,
    pub party_id: PartyId,
    pub party_address: String,
    pub threshold: u16,
    pub total_parties: u16,
}

/// Sign Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignRequest {
    pub session_id: SessionId,
    pub party_id: PartyId,
    pub message: String,
    pub participating_parties: Vec<PartyId>,
}

/// Sign Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignResponse {
    pub signature: Signature<Secp256k1>,
    pub message: String,
    pub public_key: Point<Secp256k1>,
}

/// Key Share Storage
#[derive(Clone, Serialize, Deserialize)]
pub struct StoredKeyShare {
    pub party_id: PartyId,
    pub key_share: Valid<DirtyIncompleteKeyShare<Secp256k1>>,
    pub session_id: SessionId,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Presignature Storage
#[derive(Clone, Serialize, Deserialize)]
pub struct StoredPresignature {
    pub session_id: SessionId,
    pub presignature: Presignature<Secp256k1>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Partial Signature Storage
#[derive(Clone, Serialize, Deserialize)]
pub struct StoredPartialSignature {
    pub session_id: SessionId,
    pub party_id: PartyId,
    pub partial_signature: PartialSignature<Secp256k1>,
    pub message_hash: String, // Store as string instead of DataToSign
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl SessionId {
    pub fn new() -> Self {
        let mut execution_id = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut execution_id);
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            execution_id,
        }
    }
    
    pub fn execution_id(&self) -> ExecutionId {
        ExecutionId::new(&self.execution_id)
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
} 