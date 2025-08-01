use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::fs;
use std::path::Path;
use anyhow::Result;
use serde_json;
use chrono;

use crate::types::*;
use cggmp21::{
    supported_curves::Secp256k1,
    key_share::{DirtyIncompleteKeyShare, Valid},
    signing::{Presignature, PartialSignature, DataToSign}
};

/// Storage implementation for MPC data
#[derive(Clone)]
pub struct Storage {
    data_dir: String,
    key_shares: Arc<RwLock<HashMap<String, StoredKeyShare>>>,
    presignatures: Arc<RwLock<HashMap<String, StoredPresignature>>>,
    partial_signatures: Arc<RwLock<HashMap<String, StoredPartialSignature>>>,
    sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
    // New DKG storage
    dkg_sessions: Arc<RwLock<HashMap<String, DkgSessionState>>>,
    dkg_messages: Arc<RwLock<HashMap<String, Vec<DkgMessage>>>>,
}

impl Storage {
    pub fn new(data_dir: &str) -> Result<Self> {
        fs::create_dir_all(data_dir)?;
        fs::create_dir_all(format!("{}/key_shares", data_dir))?;
        fs::create_dir_all(format!("{}/presignatures", data_dir))?;
        fs::create_dir_all(format!("{}/partial_signatures", data_dir))?;
        fs::create_dir_all(format!("{}/sessions", data_dir))?;
        fs::create_dir_all(format!("{}/dkg_sessions", data_dir))?;
        fs::create_dir_all(format!("{}/dkg_messages", data_dir))?;
        
        Ok(Self {
            data_dir: data_dir.to_string(),
            key_shares: Arc::new(RwLock::new(HashMap::new())),
            presignatures: Arc::new(RwLock::new(HashMap::new())),
            partial_signatures: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            dkg_sessions: Arc::new(RwLock::new(HashMap::new())),
            dkg_messages: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    // Session management
    pub fn create_session(&self, session_info: SessionInfo) -> Result<()> {
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session_info.session_id.id.clone(), session_info.clone());
        
        // Save to disk
        let file_path = format!("{}/sessions/{}.json", self.data_dir, session_info.session_id.id);
        let serialized = serde_json::to_string_pretty(&session_info)?;
        fs::write(file_path, serialized)?;
        
        Ok(())
    }
    
    pub fn get_session(&self, session_id: &str) -> Option<SessionInfo> {
        let sessions = self.sessions.read().unwrap();
        sessions.get(session_id).cloned()
    }
    
    pub fn update_session(&self, session_info: SessionInfo) -> Result<()> {
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session_info.session_id.id.clone(), session_info.clone());
        
        // Save to disk
        let file_path = format!("{}/sessions/{}.json", self.data_dir, session_info.session_id.id);
        let serialized = serde_json::to_string_pretty(&session_info)?;
        fs::write(file_path, serialized)?;
        
        Ok(())
    }
    
    pub fn list_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.read().unwrap();
        sessions.values().cloned().collect()
    }
    
    // Key share management
    pub fn save_key_share(&self, key_share: StoredKeyShare) -> Result<()> {
        let mut key_shares = self.key_shares.write().unwrap();
        let key = format!("{}_{}", key_share.session_id.id, key_share.party_id);
        key_shares.insert(key.clone(), key_share.clone());
        
        // Save to disk
        let file_path = format!("{}/key_shares/{}.json", self.data_dir, key);
        let serialized = serde_json::to_string_pretty(&key_share)?;
        fs::write(file_path, serialized)?;
        
        Ok(())
    }
    
    pub fn get_key_share(&self, session_id: &str, party_id: PartyId) -> Option<StoredKeyShare> {
        let key_shares = self.key_shares.read().unwrap();
        let key = format!("{}_{}", session_id, party_id);
        key_shares.get(&key).cloned()
    }
    
    pub fn list_key_shares(&self, session_id: &str) -> Vec<StoredKeyShare> {
        let key_shares = self.key_shares.read().unwrap();
        key_shares
            .values()
            .filter(|ks| ks.session_id.id == session_id)
            .cloned()
            .collect()
    }
    
    // Presignature management
    pub fn save_presignature(&self, presignature: StoredPresignature) -> Result<()> {
        let mut presignatures = self.presignatures.write().unwrap();
        presignatures.insert(presignature.session_id.id.clone(), presignature.clone());
        
        // Save to disk
        let file_path = format!("{}/presignatures/{}.json", self.data_dir, presignature.session_id.id);
        let serialized = serde_json::to_string_pretty(&presignature)?;
        fs::write(file_path, serialized)?;
        
        Ok(())
    }
    
    pub fn get_presignature(&self, session_id: &str) -> Option<StoredPresignature> {
        let presignatures = self.presignatures.read().unwrap();
        presignatures.get(session_id).cloned()
    }
    
    // Partial signature management
    pub fn save_partial_signature(&self, partial_signature: StoredPartialSignature) -> Result<()> {
        let mut partial_signatures = self.partial_signatures.write().unwrap();
        let key = format!("{}_{}", partial_signature.session_id.id, partial_signature.party_id);
        partial_signatures.insert(key.clone(), partial_signature.clone());
        
        // Save to disk
        let file_path = format!("{}/partial_signatures/{}.json", self.data_dir, key);
        let serialized = serde_json::to_string_pretty(&partial_signature)?;
        fs::write(file_path, serialized)?;
        
        Ok(())
    }
    
    pub fn get_partial_signatures(&self, session_id: &str) -> Vec<StoredPartialSignature> {
        let partial_signatures = self.partial_signatures.read().unwrap();
        partial_signatures
            .values()
            .filter(|ps| ps.session_id.id == session_id)
            .cloned()
            .collect()
    }
    
    // DKG Session methods
    pub fn save_dkg_session(&self, session: DkgSessionState) -> Result<()> {
        let key = format!("{}", session.session_id.id);
        let mut sessions = self.dkg_sessions.write().unwrap();
        sessions.insert(key.clone(), session.clone());
        
        let file_path = format!("{}/dkg_sessions/{}.json", self.data_dir, key);
        let json = serde_json::to_string_pretty(&session)?;
        fs::write(file_path, json)?;
        
        Ok(())
    }
    
    pub fn get_dkg_session(&self, session_id: &str) -> Option<DkgSessionState> {
        let sessions = self.dkg_sessions.read().unwrap();
        sessions.get(session_id).cloned()
    }
    
    pub fn list_dkg_sessions(&self) -> Vec<DkgSessionState> {
        let sessions = self.dkg_sessions.read().unwrap();
        sessions.values().cloned().collect()
    }
    
    // DKG Message methods
    pub fn save_dkg_message(&self, message: DkgMessage) -> Result<()> {
        let key = format!("{}", message.session_id.id);
        let mut messages = self.dkg_messages.write().unwrap();
        let session_messages = messages.entry(key.clone()).or_insert_with(Vec::new);
        session_messages.push(message.clone());
        
        let file_path = format!("{}/dkg_messages/{}.json", self.data_dir, key);
        let json = serde_json::to_string_pretty(&session_messages)?;
        fs::write(file_path, json)?;
        
        Ok(())
    }
    
    pub fn get_dkg_messages(&self, session_id: &str) -> Vec<DkgMessage> {
        let messages = self.dkg_messages.read().unwrap();
        messages.get(session_id).cloned().unwrap_or_default()
    }
    
    pub fn get_dkg_messages_for_round(&self, session_id: &str, round: u32) -> Vec<DkgMessage> {
        let messages = self.get_dkg_messages(session_id);
        messages.into_iter().filter(|msg| msg.round == round).collect()
    }
    
    pub fn get_dkg_messages_for_party(&self, session_id: &str, party_id: PartyId) -> Vec<DkgMessage> {
        let messages = self.get_dkg_messages(session_id);
        messages.into_iter().filter(|msg| msg.from_party == party_id).collect()
    }
    
    // Load all data from disk on startup
    pub fn load_from_disk(&self) -> Result<()> {
        self.load_sessions()?;
        self.load_key_shares()?;
        self.load_presignatures()?;
        self.load_partial_signatures()?;
        self.load_dkg_sessions()?;
        self.load_dkg_messages()?;
        Ok(())
    }
    
    fn load_sessions(&self) -> Result<()> {
        let sessions_dir = format!("{}/sessions", self.data_dir);
        if !Path::new(&sessions_dir).exists() {
            return Ok(());
        }
        
        let mut sessions = self.sessions.write().unwrap();
        
        for entry in fs::read_dir(&sessions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                let session_info: SessionInfo = serde_json::from_str(&content)?;
                sessions.insert(session_info.session_id.id.clone(), session_info);
            }
        }
        
        Ok(())
    }
    
    fn load_key_shares(&self) -> Result<()> {
        let key_shares_dir = format!("{}/key_shares", self.data_dir);
        if !Path::new(&key_shares_dir).exists() {
            return Ok(());
        }
        
        let mut key_shares = self.key_shares.write().unwrap();
        
        for entry in fs::read_dir(&key_shares_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                let stored_key_share: StoredKeyShare = serde_json::from_str(&content)?;
                let key = format!("{}_{}", stored_key_share.session_id.id, stored_key_share.party_id);
                key_shares.insert(key, stored_key_share);
            }
        }
        
        Ok(())
    }
    
    fn load_presignatures(&self) -> Result<()> {
        let presignatures_dir = format!("{}/presignatures", self.data_dir);
        if !Path::new(&presignatures_dir).exists() {
            return Ok(());
        }
        
        let mut presignatures = self.presignatures.write().unwrap();
        
        for entry in fs::read_dir(&presignatures_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                let stored_presignature: StoredPresignature = serde_json::from_str(&content)?;
                presignatures.insert(stored_presignature.session_id.id.clone(), stored_presignature);
            }
        }
        
        Ok(())
    }
    
    fn load_partial_signatures(&self) -> Result<()> {
        let partial_signatures_dir = format!("{}/partial_signatures", self.data_dir);
        if !Path::new(&partial_signatures_dir).exists() {
            return Ok(());
        }
        
        let mut partial_signatures = self.partial_signatures.write().unwrap();
        
        for entry in fs::read_dir(&partial_signatures_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                let stored_partial_signature: StoredPartialSignature = serde_json::from_str(&content)?;
                let key = format!("{}_{}", stored_partial_signature.session_id.id, stored_partial_signature.party_id);
                partial_signatures.insert(key, stored_partial_signature);
            }
        }
        
        Ok(())
    }

    fn load_dkg_sessions(&self) -> Result<()> {
        let dkg_sessions_dir = format!("{}/dkg_sessions", self.data_dir);
        if !Path::new(&dkg_sessions_dir).exists() {
            return Ok(());
        }

        let mut dkg_sessions = self.dkg_sessions.write().unwrap();

        for entry in fs::read_dir(&dkg_sessions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                let session: DkgSessionState = serde_json::from_str(&content)?;
                dkg_sessions.insert(session.session_id.id.clone(), session);
            }
        }

        Ok(())
    }

    fn load_dkg_messages(&self) -> Result<()> {
        let dkg_messages_dir = format!("{}/dkg_messages", self.data_dir);
        if !Path::new(&dkg_messages_dir).exists() {
            return Ok(());
        }

        let mut dkg_messages = self.dkg_messages.write().unwrap();

        for entry in fs::read_dir(&dkg_messages_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                let messages: Vec<DkgMessage> = serde_json::from_str(&content)?;
                dkg_messages.insert(path.parent().unwrap().file_name().unwrap().to_str().unwrap().to_string(), messages);
            }
        }

        Ok(())
    }
} 