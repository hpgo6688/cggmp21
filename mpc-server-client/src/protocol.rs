use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::types::*;
use crate::storage::Storage;

/// Protocol message wrapper for HTTP communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    pub session_id: SessionId,
    pub party_id: PartyId,
    pub round: u32,
    pub message_type: String,
    pub data: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Protocol session state
#[derive(Debug, Clone)]
pub struct ProtocolSession {
    pub session_id: SessionId,
    pub protocol_type: ProtocolType,
    pub state: ProtocolState,
    pub parties: Vec<PartyInfo>,
    pub threshold: u16,
    pub total_parties: u16,
    pub current_round: u32,
    pub messages: HashMap<u32, Vec<ProtocolMessage>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl ProtocolSession {
    pub fn new(
        session_id: SessionId,
        protocol_type: ProtocolType,
        parties: Vec<PartyInfo>,
        threshold: u16,
    ) -> Self {
        let now = chrono::Utc::now();
        let total_parties = parties.len() as u16;
        Self {
            session_id,
            protocol_type,
            state: ProtocolState::Initialized,
            parties,
            threshold,
            total_parties,
            current_round: 0,
            messages: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn add_message(&mut self, message: ProtocolMessage) {
        self.messages.entry(message.round).or_insert_with(Vec::new).push(message);
        self.updated_at = chrono::Utc::now();
    }
    
    pub fn get_round_messages(&self, round: u32) -> Vec<ProtocolMessage> {
        self.messages.get(&round).cloned().unwrap_or_default()
    }
    
    pub fn is_round_complete(&self, round: u32) -> bool {
        if let Some(messages) = self.messages.get(&round) {
            messages.len() >= self.total_parties as usize
        } else {
            false
        }
    }
}

/// Protocol manager for handling MPC sessions
pub struct ProtocolManager {
    sessions: Arc<RwLock<HashMap<String, ProtocolSession>>>,
    storage: Arc<Storage>,
}

impl ProtocolManager {
    pub fn new(storage: Storage) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            storage: Arc::new(storage),
        }
    }
    
    pub fn create_session(
        &self,
        session_id: SessionId,
        protocol_type: ProtocolType,
        parties: Vec<PartyInfo>,
        threshold: u16,
    ) -> Result<()> {
        let session = ProtocolSession::new(session_id.clone(), protocol_type, parties, threshold);
        
        // Store session info
        let session_info = SessionInfo {
            session_id: session_id.clone(),
            protocol_type: session.protocol_type.clone(),
            state: session.state.clone(),
            parties: session.parties.clone(),
            threshold: session.threshold,
            total_parties: session.total_parties,
            created_at: session.created_at,
            updated_at: session.updated_at,
        };
        
        self.storage.create_session(session_info)?;
        
        // Add to active sessions
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session_id.id, session);
        
        Ok(())
    }
    
    pub fn get_session(&self, session_id: &str) -> Option<ProtocolSession> {
        let sessions = self.sessions.read().unwrap();
        sessions.get(session_id).cloned()
    }
    
    pub fn add_message(&self, message: ProtocolMessage) -> Result<()> {
        let mut sessions = self.sessions.write().unwrap();
        
        if let Some(session) = sessions.get_mut(&message.session_id.id) {
            session.add_message(message.clone());
            
            // Update session info in storage
            let session_info = SessionInfo {
                session_id: session.session_id.clone(),
                protocol_type: session.protocol_type.clone(),
                state: session.state.clone(),
                parties: session.parties.clone(),
                threshold: session.threshold,
                total_parties: session.total_parties,
                created_at: session.created_at,
                updated_at: session.updated_at,
            };
            
            self.storage.update_session(session_info)?;
        }
        
        Ok(())
    }
    
    pub fn get_round_messages(&self, session_id: &str, round: u32) -> Vec<ProtocolMessage> {
        if let Some(session) = self.get_session(session_id) {
            session.get_round_messages(round)
        } else {
            Vec::new()
        }
    }
    
    pub fn is_round_complete(&self, session_id: &str, round: u32) -> bool {
        if let Some(session) = self.get_session(session_id) {
            session.is_round_complete(round)
        } else {
            false
        }
    }
    
    pub fn get_storage(&self) -> &Storage {
        &self.storage
    }
} 