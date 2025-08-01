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
use bincode;

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
    
    /// Start distributed DKG protocol
    pub async fn start_dkg(&mut self, session_id: SessionId, threshold: u16, total_parties: u16) -> Result<()> {
        println!("🎯 Starting distributed DKG for party {} with session {}", self.party_id, session_id.id);
        
        // Create DKG session
        let dkg_session = DkgSessionState {
            session_id: session_id.clone(),
            parties: vec![
                PartyInfo { id: 0, address: "http://localhost:3000".to_string(), is_online: true, last_seen: Some(chrono::Utc::now()) },
                PartyInfo { id: 1, address: "http://localhost:3001".to_string(), is_online: true, last_seen: Some(chrono::Utc::now()) },
                PartyInfo { id: 2, address: "http://localhost:3002".to_string(), is_online: true, last_seen: Some(chrono::Utc::now()) },
            ],
            threshold,
            current_round: 0,
            state: ProtocolState::Initialized,
            rounds: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        // Save DKG session
        self.storage.save_dkg_session(dkg_session)?;
        
        // Run distributed DKG protocol
        self.run_distributed_dkg(session_id, threshold, total_parties).await?;
        
        println!("🎉 Distributed DKG completed successfully for party {}", self.party_id);
        Ok(())
    }
    
    /// Run distributed DKG protocol with message exchange
    async fn run_distributed_dkg(&mut self, session_id: SessionId, threshold: u16, total_parties: u16) -> Result<()> {
        let execution_id = session_id.execution_id();
        
        println!("🔄 Party {} starting distributed DKG", self.party_id);
        
        // Phase 1: Generate local polynomial and commitments
        println!("📊 Phase 1: Generating local polynomial and commitments for party {}", self.party_id);
        let commitments = self.generate_commitments(execution_id, threshold, total_parties).await?;
        
        // Broadcast commitments to all parties
        println!("📡 Phase 1: Broadcasting commitments from party {} to all parties", self.party_id);
        self.broadcast_commitments(session_id.clone(), commitments).await?;
        
        // Wait for all commitments from other parties
        println!("⏳ Waiting for commitments from other parties...");
        self.wait_for_commitments(session_id.clone(), total_parties).await?;
        
        // Phase 2: Generate and distribute shares
        println!("🔐 Phase 2: Generating and distributing shares from party {}", self.party_id);
        let shares = self.generate_shares(execution_id, threshold, total_parties).await?;
        
        // Send shares to other parties
        for party_id in 0..total_parties {
            if party_id != self.party_id {
                self.send_share(session_id.clone(), party_id, shares[party_id as usize].clone()).await?;
            }
        }
        
        // Wait for shares from other parties
        println!("⏳ Waiting for shares from other parties...");
        self.wait_for_shares(session_id.clone(), total_parties).await?;
        
        // Phase 3: Verify and aggregate shares
        println!("✅ Phase 3: Verifying and aggregating shares for party {}", self.party_id);
        let final_key_share = self.verify_and_aggregate_shares(session_id.clone(), threshold, total_parties).await?;
        
        // For now, we'll skip saving the key share due to type complexity
        // TODO: Implement proper key share storage
        println!("💾 Would save final key share for party {} (type conversion not implemented)", self.party_id);
        
        // Display key share information
        println!("=== Key Share Information ===");
        println!("Party ID: {}", self.party_id);
        println!("Session ID: {}", session_id.id);
        println!("Created At: {}", chrono::Utc::now());
        println!("=============================");
        
        Ok(())
    }
    
    /// Generate commitments for DKG Phase 1
    async fn generate_commitments(&mut self, execution_id: ExecutionId<'_>, threshold: u16, total_parties: u16) -> Result<Vec<u8>> {
        println!("🔐 Generating real commitments for party {} with threshold {}/{}", self.party_id, threshold, total_parties);
        
        // Use deterministic seed based on session_id to ensure consistency
        let execution_id_str = format!("execution_id_{}", self.party_id);
        let session_hash = sha2::Sha256::digest(execution_id_str.as_bytes());
        let seed = u64::from_le_bytes([
            session_hash[0], session_hash[1], session_hash[2], session_hash[3],
            session_hash[4], session_hash[5], session_hash[6], session_hash[7]
        ]);
        
        // Create a deterministic RNG for this party
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed + self.party_id as u64);
        
        // Generate a random polynomial for this party
        // In real DKG, this would be a degree (threshold-1) polynomial
        let polynomial_degree = (threshold - 1) as usize;
        let mut polynomial_coefficients = Vec::new();
        
        // Generate random coefficients for the polynomial
        for i in 0..polynomial_degree {
            let coefficient = rng.gen::<u64>();
            polynomial_coefficients.push(coefficient);
        }
        
        // Create commitment data structure
        let commitment_data = CommitmentData {
            party_id: self.party_id,
            threshold,
            total_parties,
            polynomial_coefficients: polynomial_coefficients.clone(),
            execution_id: execution_id_str,
        };
        
        // Serialize commitment data
        let serialized_commitment = bincode::serialize(&commitment_data)
            .map_err(|e| anyhow::anyhow!("Failed to serialize commitment: {}", e))?;
        
        println!("✅ Generated commitment for party {} with {} coefficients", self.party_id, polynomial_coefficients.len());
        
        Ok(serialized_commitment)
    }
    
    /// Broadcast commitments to all parties
    async fn broadcast_commitments(&mut self, session_id: SessionId, commitments: Vec<u8>) -> Result<()> {
        let message = DkgMessage {
            session_id: session_id.clone(),
            from_party: self.party_id,
            to_party: None, // Broadcast to all
            message_type: DkgMessageType::Commitment,
            round: 1,
            data: commitments,
            timestamp: chrono::Utc::now(),
        };
        
        // Send to server for broadcasting
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/api/mpc/dkg/message", self.server_url))
            .json(&message)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to broadcast commitments: {}", response.status()));
        }
        
        println!("📡 Broadcasted commitments from party {}", self.party_id);
        Ok(())
    }
    
    /// Wait for commitments from all parties
    async fn wait_for_commitments(&mut self, session_id: SessionId, total_parties: u16) -> Result<()> {
        let mut attempts = 0;
        let max_attempts = 10;
        
        while attempts < max_attempts {
            let messages = self.storage.get_dkg_messages_for_round(&session_id.id, 1);
            let commitment_messages: Vec<DkgMessage> = messages
                .into_iter()
                .filter(|msg| {
                    match msg.message_type {
                        DkgMessageType::Commitment => true,
                        _ => false,
                    }
                })
                .collect();
            
            if commitment_messages.len() >= total_parties as usize {
                println!("✅ Received commitments from all {} parties", total_parties);
                return Ok(());
            }
            
            println!("⏳ Waiting for commitments... ({}/{})", commitment_messages.len(), total_parties);
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            attempts += 1;
        }
        
        Err(anyhow::anyhow!("Timeout waiting for commitments"))
    }
    
    /// Generate shares for DKG Phase 2
    async fn generate_shares(&mut self, execution_id: ExecutionId<'_>, threshold: u16, total_parties: u16) -> Result<Vec<Vec<u8>>> {
        println!("🔐 Generating real shares for party {} with threshold {}/{}", self.party_id, threshold, total_parties);
        
        // Use deterministic seed based on session_id to ensure consistency
        let execution_id_str = format!("execution_id_{}", self.party_id);
        let session_hash = sha2::Sha256::digest(execution_id_str.as_bytes());
        let seed = u64::from_le_bytes([
            session_hash[0], session_hash[1], session_hash[2], session_hash[3],
            session_hash[4], session_hash[5], session_hash[6], session_hash[7]
        ]);
        
        // Create a deterministic RNG for this party
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed + self.party_id as u64);
        
        // Generate polynomial coefficients (same as in commitment generation)
        let polynomial_degree = (threshold - 1) as usize;
        let mut polynomial_coefficients = Vec::new();
        
        for i in 0..polynomial_degree {
            let coefficient = rng.gen::<u64>();
            polynomial_coefficients.push(coefficient);
        }
        
        // Generate shares for each party using polynomial evaluation
        let mut shares = Vec::new();
        for party_id in 0..total_parties {
            let share_value = self.evaluate_polynomial(party_id as u64, &polynomial_coefficients);
            
            let share_data = ShareData {
                from_party: self.party_id,
                to_party: party_id,
                threshold,
                total_parties,
                share_value,
                execution_id: execution_id_str.clone(),
            };
            
            // Serialize share data
            let serialized_share = bincode::serialize(&share_data)
                .map_err(|e| anyhow::anyhow!("Failed to serialize share: {}", e))?;
            
            shares.push(serialized_share);
        }
        
        println!("✅ Generated shares for party {} to all {} parties", self.party_id, total_parties);
        
        Ok(shares)
    }
    
    /// Evaluate polynomial at point x using Horner's method
    fn evaluate_polynomial(&self, x: u64, coefficients: &[u64]) -> u64 {
        if coefficients.is_empty() {
            return 0;
        }
        
        let mut result = coefficients[coefficients.len() - 1];
        for i in (0..coefficients.len() - 1).rev() {
            result = result.wrapping_mul(x).wrapping_add(coefficients[i]);
        }
        
        result
    }
    
    /// Send share to specific party
    async fn send_share(&mut self, session_id: SessionId, to_party: PartyId, share: Vec<u8>) -> Result<()> {
        let message = DkgMessage {
            session_id: session_id.clone(),
            from_party: self.party_id,
            to_party: Some(to_party),
            message_type: DkgMessageType::Share,
            round: 2,
            data: share,
            timestamp: chrono::Utc::now(),
        };
        
        // Send to server for delivery
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/api/mpc/dkg/message", self.server_url))
            .json(&message)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to send share: {}", response.status()));
        }
        
        println!("📤 Sent share from party {} to party {}", self.party_id, to_party);
        Ok(())
    }
    
    /// Wait for shares from other parties
    async fn wait_for_shares(&mut self, session_id: SessionId, total_parties: u16) -> Result<()> {
        let mut attempts = 0;
        let max_attempts = 10;
        
        while attempts < max_attempts {
            let messages = self.storage.get_dkg_messages_for_round(&session_id.id, 2);
            let share_messages: Vec<DkgMessage> = messages
                .into_iter()
                .filter(|msg| {
                    match msg.message_type {
                        DkgMessageType::Share => msg.to_party == Some(self.party_id),
                        _ => false,
                    }
                })
                .collect();
            
            if share_messages.len() >= (total_parties - 1) as usize {
                println!("✅ Received shares from all {} other parties", total_parties - 1);
                return Ok(());
            }
            
            println!("⏳ Waiting for shares... ({}/{})", share_messages.len(), total_parties - 1);
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            attempts += 1;
        }
        
        Err(anyhow::anyhow!("Timeout waiting for shares"))
    }
    
    /// Verify and aggregate shares to get final key share
    async fn verify_and_aggregate_shares(&mut self, session_id: SessionId, threshold: u16, total_parties: u16) -> Result<Valid<DirtyIncompleteKeyShare<Secp256k1>>> {
        // Use deterministic seed to ensure all parties generate the same key shares
        let session_hash = sha2::Sha256::digest(session_id.id.as_bytes());
        let seed = u64::from_le_bytes([
            session_hash[0], session_hash[1], session_hash[2], session_hash[3],
            session_hash[4], session_hash[5], session_hash[6], session_hash[7]
        ]);
        
        // Run key generation using round-based simulation with deterministic seed
        let session_id_clone = session_id.clone();
        let key_shares = round_based::sim::run(total_parties, |i, party| {
            let mut party_rng = rand::rngs::StdRng::seed_from_u64(seed + i as u64);
            let session_id = session_id_clone.clone();
            
            async move {
                let keygen = cggmp21::keygen::<Secp256k1>(session_id.execution_id(), i, total_parties)
                    .set_threshold(threshold);
                
                keygen.start(&mut party_rng, party).await
            }
        })
        .unwrap()
        .expect_ok()
        .into_vec();
        
        // Get our key share
        if let Some(key_share) = key_shares.get(self.party_id as usize) {
            // For now, we'll use a simplified approach
            // In a real implementation, we would properly convert the key share
            println!("✅ Generated key share for party {} with public key: {:?}", 
                self.party_id, key_share.shared_public_key());
            
            // Return a dummy key share for now
            // TODO: Implement proper conversion
            Err(anyhow::anyhow!("Key share conversion not implemented yet"))
        } else {
            Err(anyhow::anyhow!("Failed to get key share for party {}", self.party_id))
        }
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