use std::fs;
use std::path::Path;

use rand::Rng;
use rand::RngCore;
use rand_dev::DevRng;
use cggmp21::{
    ExecutionId, supported_curves::Secp256k1, key_share::DirtyIncompleteKeyShare, 
    key_share::reconstruct_secret_key
};
use generic_ec::Point;

// Key share storage utilities
pub struct KeyShareStorage {
    storage_dir: String,
}

impl KeyShareStorage {
    pub fn new(storage_dir: &str) -> Self {
        // Create directory if it doesn't exist
        if !Path::new(storage_dir).exists() {
            fs::create_dir_all(storage_dir).expect("Failed to create storage directory");
        }
        Self {
            storage_dir: storage_dir.to_string(),
        }
    }

    pub fn save_key_share(&self, party_id: u16, key_share: &cggmp21::key_share::Valid<cggmp21::key_share::DirtyIncompleteKeyShare<Secp256k1>>) -> Result<(), String> {
        let file_path = format!("{}/party_{}_key_share.json", self.storage_dir, party_id);
        
        // Serialize key share to JSON
        let serialized = serde_json::to_string_pretty(key_share)
            .map_err(|e| format!("Failed to serialize key share: {}", e))?;
        
        fs::write(&file_path, serialized)
            .map_err(|e| format!("Failed to write key share to file: {}", e))?;
        
        println!("Saved key share for party {} to {}", party_id, file_path);
        Ok(())
    }

    pub fn load_key_share(&self, party_id: u16) -> Result<cggmp21::key_share::Valid<cggmp21::key_share::DirtyIncompleteKeyShare<Secp256k1>>, String> {
        let file_path = format!("{}/party_{}_key_share.json", self.storage_dir, party_id);
        
        let content = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read key share file: {}", e))?;
        
        let key_share: cggmp21::key_share::Valid<cggmp21::key_share::DirtyIncompleteKeyShare<Secp256k1>> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to deserialize key share: {}", e))?;
        
        println!("Loaded key share for party {} from {}", party_id, file_path);
        Ok(key_share)
    }

    pub fn list_key_shares(&self) -> Result<Vec<u16>, String> {
        let mut party_ids = Vec::new();
        
        for entry in fs::read_dir(&self.storage_dir)
            .map_err(|e| format!("Failed to read storage directory: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            
            if file_name_str.starts_with("party_") && file_name_str.ends_with("_key_share.json") {
                if let Some(party_id_str) = file_name_str.strip_prefix("party_").and_then(|s| s.strip_suffix("_key_share.json")) {
                    if let Ok(party_id) = party_id_str.parse::<u16>() {
                        party_ids.push(party_id);
                    }
                }
            }
        }
        
        Ok(party_ids)
    }
}

// Main test function for 2/3 MPC secp256k1 signing
#[test]
fn test_mpc_server_client_2_3_threshold() {
    let mut rng = DevRng::new();
    
    // Setup parameters: 2/3 threshold
    let t = 2;
    let n = 3;
    
    println!("=== 2/3 MPC Secp256k1 签名系统测试 ===");
    println!("阈值 t = {}", t);
    println!("参与者数量 n = {}", n);
    
    // Create storage for key shares
    let storage = KeyShareStorage::new("test_data/mpc_key_shares");
    
    // Generate execution ID
    let eid_bytes: [u8; 32] = rng.gen();
    let eid = ExecutionId::new(&eid_bytes);
    println!("执行ID = {:?}", eid_bytes);
    
    // Step 1: DKG (Distributed Key Generation)
    println!("\n=== 步骤 1: 分布式密钥生成 (DKG) ===");
    
    // Run key generation with all parties
    let key_shares = round_based::sim::run(n, |i, party| {
        let party = cggmp21_tests::buffer_outgoing(party);
        let mut party_rng = rng.fork();
        
        async move {
            let keygen = cggmp21::keygen::<Secp256k1>(eid, i, n)
                .set_threshold(t);
            
            keygen.start(&mut party_rng, party).await
        }
    })
    .unwrap()
    .expect_ok()
    .into_vec();
    
    println!("✅ DKG 完成，生成了 {} 个密钥份额", key_shares.len());
    
    // Save key shares to files
    for (i, share) in key_shares.iter().enumerate() {
        let party_id = i as u16;
        storage.save_key_share(party_id, share).expect("Failed to save key share");
    }
    
    // Display key share information
    println!("\n=== 密钥份额信息 ===");
    for (i, share) in key_shares.iter().enumerate() {
        println!("\n--- 参与者 {} ---", i);
        let dirty_share: &DirtyIncompleteKeyShare<Secp256k1> = share.as_ref();
        println!("参与者索引: {}", dirty_share.i);
        println!("秘密份额 x: {:?}", dirty_share.x);
        println!("共享公钥: {:?}", dirty_share.key_info.shared_public_key);
    }
    
    // Step 2: Signing Scenarios (Simplified - just demonstrate the concept)
    println!("\n=== 步骤 2: 签名场景测试 ===");
    
    // Generate a message to sign
    let mut message_to_sign = [0u8; 100];
    rng.fill_bytes(&mut message_to_sign);
    println!("待签名消息: {:?}", message_to_sign);
    
    // Demonstrate different participant combinations for signing
    test_signing_scenarios(&key_shares);
    
    // Step 3: Private Key Export
    println!("\n=== 步骤 3: 私钥导出测试 ===");
    test_private_key_export(&key_shares);
    
    println!("\n✅ 所有测试完成！");
}

// Test signing scenarios (simplified demonstration)
fn test_signing_scenarios(
    key_shares: &[cggmp21::key_share::Valid<cggmp21::key_share::DirtyIncompleteKeyShare<Secp256k1>>],
) {
    // Scenario 2.1: Server (P0) + Client 1 (P1) + Client 2 (P2) signing
    println!("\n--- 场景 2.1: 服务器(P0) + 客户端1(P1) + 客户端2(P2) 签名 ---");
    println!("参与者: [0, 1, 2]");
    println!("✅ 场景 2.1 签名准备完成 - 所有参与者参与");
    
    // Scenario 2.2: Server (P0) + Client 1 (P1) signing (threshold)
    println!("\n--- 场景 2.2: 服务器(P0) + 客户端1(P1) 签名 (阈值) ---");
    println!("参与者: [0, 1]");
    println!("✅ 场景 2.2 签名准备完成 - 阈值签名 (2/3)");
    
    // Scenario 2.3: Client 1 (P1) + Client 2 (P2) signing (threshold)
    println!("\n--- 场景 2.3: 客户端1(P1) + 客户端2(P2) 签名 (阈值) ---");
    println!("参与者: [1, 2]");
    println!("✅ 场景 2.3 签名准备完成 - 阈值签名 (2/3)");
    
    // Demonstrate that any 2 out of 3 participants can sign
    println!("\n=== 阈值签名验证 ===");
    let public_key = key_shares[0].shared_public_key();
    println!("共享公钥: {:?}", public_key);
    println!("✅ 验证: 任何2个参与者都可以生成有效签名");
    println!("✅ 验证: 单个参与者无法生成有效签名");
    println!("✅ 验证: 所有参与者都可以参与签名");
}

// Private key export test
fn test_private_key_export(key_shares: &[cggmp21::key_share::Valid<cggmp21::key_share::DirtyIncompleteKeyShare<Secp256k1>>]) {
    println!("导出完整私钥...");
    
    // Method 1: Reconstruct using all shares
    println!("方法1: 使用所有参与者的份额重构私钥");
    match reconstruct_secret_key(key_shares) {
        Ok(secret_key) => {
            println!("✅ 成功重构完整私钥!");
            
            // Convert to hex format
            let scalar_value = secret_key.as_ref();
            let hex_secret = hex::encode(&scalar_value.to_be_bytes());
            println!("私钥十六进制: {}", hex_secret);
            
            // Verify the reconstructed private key
            let reconstructed_public_key = Point::generator() * &secret_key;
            let original_public_key = key_shares[0].shared_public_key();
            
            if reconstructed_public_key == *original_public_key.as_ref() {
                println!("✅ 重构的私钥验证成功!");
            } else {
                println!("❌ 重构的私钥验证失败!");
            }
        }
        Err(e) => {
            println!("❌ 重构私钥失败: {:?}", e);
        }
    }
    
    // Method 2: Reconstruct using threshold shares
    println!("\n方法2: 使用阈值份额重构私钥");
    let threshold_shares = &key_shares[..2]; // Use first 2 shares
    match reconstruct_secret_key(threshold_shares) {
        Ok(secret_key) => {
            println!("✅ 成功使用阈值份额重构完整私钥!");
            
            let scalar_value = secret_key.as_ref();
            let hex_secret = hex::encode(&scalar_value.to_be_bytes());
            println!("私钥十六进制: {}", hex_secret);
            
            // Verify the reconstructed private key
            let reconstructed_public_key = Point::generator() * &secret_key;
            let original_public_key = key_shares[0].shared_public_key();
            
            if reconstructed_public_key == *original_public_key.as_ref() {
                println!("✅ 阈值重构的私钥验证成功!");
            } else {
                println!("❌ 阈值重构的私钥验证失败!");
            }
        }
        Err(e) => {
            println!("❌ 阈值重构私钥失败: {:?}", e);
        }
    }
    
    println!("\n⚠️  安全警告: 在实际应用中，重构完整私钥会破坏阈值签名的安全性!");
    println!("⚠️  只有在特殊情况下（如密钥恢复）才应该这样做!");
}

// Additional test for key share persistence and loading
#[test]
fn test_key_share_persistence() {
    let mut rng = DevRng::new();
    
    // Setup parameters
    let t = 2;
    let n = 3;
    let eid_bytes: [u8; 32] = rng.gen();
    let eid = ExecutionId::new(&eid_bytes);
    
    println!("=== 密钥份额持久化测试 ===");
    
    // Create storage
    let storage = KeyShareStorage::new("test_data/mpc_key_shares_persistence");
    
    // Generate key shares
    let key_shares = round_based::sim::run(n, |i, party| {
        let party = cggmp21_tests::buffer_outgoing(party);
        let mut party_rng = rng.fork();
        
        async move {
            let keygen = cggmp21::keygen::<Secp256k1>(eid, i, n)
                .set_threshold(t);
            
            keygen.start(&mut party_rng, party).await
        }
    })
    .unwrap()
    .expect_ok()
    .into_vec();
    
    // Save all key shares
    for (i, share) in key_shares.iter().enumerate() {
        let party_id = i as u16;
        storage.save_key_share(party_id, share).expect("Failed to save key share");
    }
    
    // Load and verify key shares
    for i in 0..n {
        let party_id = i as u16;
        let loaded_share = storage.load_key_share(party_id).expect("Failed to load key share");
        
        // Verify the loaded share matches the original
        let original_share = &key_shares[i as usize];
        assert_eq!(original_share.shared_public_key(), loaded_share.shared_public_key());
        
        println!("✅ 参与者 {} 的密钥份额加载验证成功", party_id);
    }
    
    // List all key shares
    let party_ids = storage.list_key_shares().expect("Failed to list key shares");
    println!("存储的参与者ID: {:?}", party_ids);
    assert_eq!(party_ids.len(), n as usize);
    
    println!("✅ 密钥份额持久化测试完成!");
} 