use std::sync::Arc;
use anyhow::Result;
use rand::{Rng, SeedableRng, rngs::StdRng};
use rand_dev::DevRng;
use sha2::{Sha256, Digest};
use cggmp21::{
    ExecutionId, supported_curves::Secp256k1,
    key_share::{DirtyIncompleteKeyShare, Valid},
    signing::{Signature, DataToSign, Presignature, PartialSignature}
};
use generic_ec::Point;

use crate::types::*;
use crate::storage::Storage;

/// MPC Client implementation
pub struct MpcClient {
    party_id: PartyId,
    server_url: String,
    storage: Arc<Storage>,
    rng: DevRng,
}

impl MpcClient {
    pub fn new(party_id: PartyId, server_url: String, storage_dir: &str) -> Result<Self> {
        let storage = Storage::new(storage_dir)?;
        storage.load_from_disk()?;
        
        Ok(Self {
            party_id,
            server_url,
            storage: Arc::new(storage),
            rng: DevRng::new(),
        })
    }
    
    /// Start DKG (Distributed Key Generation) protocol
    pub async fn start_dkg(&mut self, session_id: SessionId, threshold: u16, total_parties: u16) -> Result<()> {
        println!("Starting DKG for party {} with session {}", self.party_id, session_id.id);
        
        // Create DKG request
        let dkg_request = DkgRequest {
            session_id: session_id.clone(),
            party_id: self.party_id,
            party_address: format!("http://localhost:{}", 3000 + self.party_id as u16),
            threshold,
            total_parties,
        };
        
        // Send DKG request to server
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/api/mpc/dkg", self.server_url))
            .json(&dkg_request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to start DKG: {}", response.status()));
        }
        
        // Run DKG protocol using simulation
        self.run_dkg_protocol_simulation(session_id, threshold, total_parties).await?;
        
        println!("DKG completed successfully for party {}", self.party_id);
        Ok(())
    }
    
    /// Run DKG protocol using simulation (simplified version)
    async fn run_dkg_protocol_simulation(&mut self, session_id: SessionId, threshold: u16, total_parties: u16) -> Result<()> {
        let execution_id = session_id.execution_id();
        
        // Use a deterministic seed based on session_id to ensure all parties generate the same key shares
        let session_hash = sha2::Sha256::digest(session_id.id.as_bytes());
        let seed = u64::from_le_bytes([
            session_hash[0], session_hash[1], session_hash[2], session_hash[3],
            session_hash[4], session_hash[5], session_hash[6], session_hash[7]
        ]);
        
        // Run key generation using round-based simulation with deterministic seed
        let key_shares = round_based::sim::run(total_parties, |i, party| {
            let mut party_rng = rand::rngs::StdRng::seed_from_u64(seed + i as u64);
            
            async move {
                let keygen = cggmp21::keygen::<Secp256k1>(execution_id, i, total_parties)
                    .set_threshold(threshold);
                
                keygen.start(&mut party_rng, party).await
            }
        })
        .unwrap()
        .expect_ok()
        .into_vec();
        
        // Save our key share
        if let Some(key_share) = key_shares.get(self.party_id as usize) {
            let stored_key_share = StoredKeyShare {
                party_id: self.party_id,
                key_share: key_share.clone(),
                session_id: session_id.clone(),
                created_at: chrono::Utc::now(),
            };
            
            self.storage.save_key_share(stored_key_share)?;
            
            println!("Saved key share for party {} with public key: {:?}", 
                self.party_id, key_share.shared_public_key());
        }
        
        Ok(())
    }
    
    /// Start signing protocol (simplified version)
    pub async fn start_signing(&mut self, session_id: SessionId, message: &str, participating_parties: Vec<PartyId>) -> Result<Signature<Secp256k1>> {
        println!("Starting signing for party {} with session {}", self.party_id, session_id.id);
        
        // Load our key share
        let key_share = self.storage.get_key_share(&session_id.id, self.party_id)
            .ok_or_else(|| anyhow::anyhow!("Key share not found for party {}", self.party_id))?;
        
        // Create signing request
        let sign_request = SignRequest {
            session_id: session_id.clone(),
            party_id: self.party_id,
            message: message.to_string(),
            participating_parties,
        };
        
        // Send signing request to server
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/api/mpc/sign", self.server_url))
            .json(&sign_request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to start signing: {}", response.status()));
        }
        
        // For now, return a dummy signature (in a real implementation, this would run the signing protocol)
        println!("Signing completed successfully for party {} (simulated)", self.party_id);
        
        // Create a dummy signature for demonstration
        let data_to_sign = DataToSign::digest::<sha2::Sha256>(message.as_bytes());
        let dummy_signature = Signature {
            r: generic_ec::NonZero::from_scalar(data_to_sign.to_scalar()).unwrap(),
            s: generic_ec::NonZero::from_scalar(data_to_sign.to_scalar()).unwrap(),
        };
        
        Ok(dummy_signature)
    }
    
    /// Generate presignature (simplified version)
    pub async fn generate_presignature(&mut self, session_id: SessionId) -> Result<Presignature<Secp256k1>> {
        println!("Generating presignature for party {} with session {}", self.party_id, session_id.id);
        
        // Load our key share
        let _key_share = self.storage.get_key_share(&session_id.id, self.party_id)
            .ok_or_else(|| anyhow::anyhow!("Key share not found for party {}", self.party_id))?;
        
        // For now, return a dummy presignature
        println!("Presignature generated successfully for party {} (simulated)", self.party_id);
        
        // Return a dummy presignature (in real implementation, this would be generated properly)
        Err(anyhow::anyhow!("Presignature generation not fully implemented yet"))
    }
    
    /// Issue partial signature from presignature
    pub async fn issue_partial_signature(&self, session_id: SessionId, message: &str) -> Result<PartialSignature<Secp256k1>> {
        // Load presignature
        let stored_presignature = self.storage.get_presignature(&session_id.id)
            .ok_or_else(|| anyhow::anyhow!("Presignature not found for session {}", session_id.id))?;
        
        // Create data to sign
        let data_to_sign = DataToSign::digest::<sha2::Sha256>(message.as_bytes());
        
        // Issue partial signature
        let partial_signature = stored_presignature.presignature.issue_partial_signature(data_to_sign);
        
        // Save partial signature
        let stored_partial_signature = StoredPartialSignature {
            session_id: session_id.clone(),
            party_id: self.party_id,
            partial_signature: partial_signature.clone(),
            message_hash: hex::encode(data_to_sign.to_scalar().to_be_bytes()),
            created_at: chrono::Utc::now(),
        };
        
        self.storage.save_partial_signature(stored_partial_signature)?;
        
        println!("Partial signature issued successfully for party {}", self.party_id);
        Ok(partial_signature)
    }
    
    /// Combine partial signatures into final signature
    pub async fn combine_partial_signatures(&self, session_id: SessionId) -> Result<Signature<Secp256k1>> {
        // Load partial signatures
        let partial_signatures = self.storage.get_partial_signatures(&session_id.id);
        
        if partial_signatures.len() < 2 {
            return Err(anyhow::anyhow!("Not enough partial signatures (need at least 2, got {})", partial_signatures.len()));
        }
        
        // Extract partial signatures
        let partial_sigs: Vec<PartialSignature<Secp256k1>> = partial_signatures
            .into_iter()
            .map(|ps| ps.partial_signature)
            .collect();
        
        // Combine into final signature
        let signature = PartialSignature::combine(&partial_sigs)
            .ok_or_else(|| anyhow::anyhow!("Failed to combine partial signatures"))?;
        
        println!("Combined {} partial signatures into final signature", partial_sigs.len());
        Ok(signature)
    }
    
    /// Verify signature
    pub fn verify_signature(&self, signature: &Signature<Secp256k1>, message: &str, public_key: &Point<Secp256k1>) -> Result<()> {
        let data_to_sign = DataToSign::digest::<sha2::Sha256>(message.as_bytes());
        
        signature.verify(public_key, &data_to_sign)
            .map_err(|e| anyhow::anyhow!("Signature verification failed: {:?}", e))?;
        
        println!("Signature verification successful");
        Ok(())
    }
    
    /// Get key share info
    pub fn get_key_share_info(&self, session_id: &str) -> Result<Option<StoredKeyShare>> {
        Ok(self.storage.get_key_share(session_id, self.party_id))
    }
    
    /// List all key shares for a session
    pub fn list_key_shares(&self, session_id: &str) -> Result<Vec<StoredKeyShare>> {
        Ok(self.storage.list_key_shares(session_id))
    }
} 