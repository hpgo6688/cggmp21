use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
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

/// DKG Message types for interactive protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DkgMessageType {
    /// Phase 1: Commitment broadcast
    Commitment,
    /// Phase 2: Share distribution (encrypted)
    Share,
    /// Phase 3: Verification
    Verification,
    /// Phase 4: Final confirmation
    Confirmation,
}

/// DKG Message for interactive protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DkgMessage {
    pub session_id: SessionId,
    pub from_party: PartyId,
    pub to_party: Option<PartyId>, // None for broadcast messages
    pub message_type: DkgMessageType,
    pub round: u32,
    pub data: Vec<u8>, // Serialized message data
    pub timestamp: DateTime<Utc>,
}

/// DKG Round state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DkgRoundState {
    pub session_id: SessionId,
    pub round: u32,
    pub messages: Vec<DkgMessage>,
    pub completed_parties: Vec<PartyId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// DKG Session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DkgSessionState {
    pub session_id: SessionId,
    pub parties: Vec<PartyInfo>,
    pub threshold: u16,
    pub current_round: u32,
    pub state: ProtocolState,
    pub rounds: Vec<DkgRoundState>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// DKG Commitment Data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentData {
    pub party_id: PartyId,
    pub threshold: u16,
    pub total_parties: u16,
    pub polynomial_coefficients: Vec<u64>,
    pub execution_id: String,
}

/// DKG Share Data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareData {
    pub from_party: PartyId,
    pub to_party: PartyId,
    pub threshold: u16,
    pub total_parties: u16,
    pub share_value: u64,
    pub execution_id: String,
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