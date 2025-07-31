use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use anyhow::Result;
use serde_json;
use std::fs;
use std::path::Path;

use crate::types::*;
use cggmp21::{
    supported_curves::Secp256k1,
    key_share::{DirtyIncompleteKeyShare, Valid},
    signing::{Presignature, PartialSignature, DataToSign}
};

/// Storage manager for MPC data
#[derive(Clone)]
pub struct Storage {
    sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
    key_shares: Arc<RwLock<HashMap<String, StoredKeyShare>>>,
    presignatures: Arc<RwLock<HashMap<String, StoredPresignature>>>,
    partial_signatures: Arc<RwLock<HashMap<String, StoredPartialSignature>>>,
    storage_dir: String,
}

impl Storage {
    pub fn new(storage_dir: &str) -> Result<Self> {
        // Create storage directory if it doesn't exist
        if !Path::new(storage_dir).exists() {
            fs::create_dir_all(storage_dir)?;
        }
        
        let sessions_dir = format!("{}/sessions", storage_dir);
        let key_shares_dir = format!("{}/key_shares", storage_dir);
        let presignatures_dir = format!("{}/presignatures", storage_dir);
        let partial_signatures_dir = format!("{}/partial_signatures", storage_dir);
        
        for dir in &[&sessions_dir, &key_shares_dir, &presignatures_dir, &partial_signatures_dir] {
            if !Path::new(dir).exists() {
                fs::create_dir_all(dir)?;
            }
        }
        
        Ok(Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            key_shares: Arc::new(RwLock::new(HashMap::new())),
            presignatures: Arc::new(RwLock::new(HashMap::new())),
            partial_signatures: Arc::new(RwLock::new(HashMap::new())),
            storage_dir: storage_dir.to_string(),
        })
    }
    
    // Session management
    pub fn create_session(&self, session_info: SessionInfo) -> Result<()> {
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session_info.session_id.id.clone(), session_info.clone());
        
        // Save to disk
        let file_path = format!("{}/sessions/{}.json", self.storage_dir, session_info.session_id.id);
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
        let file_path = format!("{}/sessions/{}.json", self.storage_dir, session_info.session_id.id);
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
        let file_path = format!("{}/key_shares/{}.json", self.storage_dir, key);
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
        let file_path = format!("{}/presignatures/{}.json", self.storage_dir, presignature.session_id.id);
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
        let file_path = format!("{}/partial_signatures/{}.json", self.storage_dir, key);
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
    
    // Load all data from disk on startup
    pub fn load_from_disk(&self) -> Result<()> {
        self.load_sessions()?;
        self.load_key_shares()?;
        self.load_presignatures()?;
        self.load_partial_signatures()?;
        Ok(())
    }
    
    fn load_sessions(&self) -> Result<()> {
        let sessions_dir = format!("{}/sessions", self.storage_dir);
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
        let key_shares_dir = format!("{}/key_shares", self.storage_dir);
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
        let presignatures_dir = format!("{}/presignatures", self.storage_dir);
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
        let partial_signatures_dir = format!("{}/partial_signatures", self.storage_dir);
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
} 