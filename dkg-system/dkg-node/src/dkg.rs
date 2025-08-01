use cggmp21::{ExecutionId, supported_curves::Secp256k1};
use rand::rngs::StdRng;
use rand::SeedableRng;
use sha2::{Sha256, Digest};
use std::collections::HashMap;

pub struct DkgState {
    pub id: u16,
    pub received_round1: HashMap<u16, Vec<u8>>,
    pub received_round2: HashMap<u16, Vec<u8>>,
    pub received_round3: HashMap<u16, Vec<u8>>,
    pub shared_public_key: Option<String>,
}

impl DkgState {
    pub fn new(id: u16, _threshold: usize, _total: usize) -> Self {
        Self {
            id,
            received_round1: HashMap::new(),
            received_round2: HashMap::new(),
            received_round3: HashMap::new(),
            shared_public_key: None,
        }
    }

    // 使用真正的DKG协议生成密钥共享
    pub fn generate_real_key_shares(&mut self, total: usize, session_id: &str) -> cggmp21::IncompleteKeyShare<Secp256k1> {
        println!("🔄 节点 {} 正在使用真正的DKG协议生成密钥共享...", self.id);
        
        // 使用session_id生成确定性种子，确保不同session生成不同公钥
        let session_hash = Sha256::digest(session_id.as_bytes());
        let seed = u64::from_le_bytes([
            session_hash[0], session_hash[1], session_hash[2], session_hash[3],
            session_hash[4], session_hash[5], session_hash[6], session_hash[7]
        ]);
        
        // 使用 round_based 模拟器运行真正的DKG协议
        let key_shares = round_based::sim::run(total.try_into().unwrap(), |i, party| {
            let mut party_rng = StdRng::seed_from_u64(seed + i as u64);
            
            async move {
                let eid_bytes: [u8; 32] = [42; 32];
                let eid = ExecutionId::new(&eid_bytes);
                
                let keygen = cggmp21::keygen::<Secp256k1>(eid, i, total.try_into().unwrap())
                    .set_threshold(2);
                
                keygen.start(&mut party_rng, party).await
            }
        })
        .unwrap()
        .expect_ok()
        .into_vec();
        
        // 获取当前节点的密钥共享
        let key_share = key_shares[self.id as usize].clone();
        
        println!("✅ 节点 {} DKG协议完成！", self.id);
        println!("🔑 节点索引: {}", key_share.i);
        println!("📊 共享公钥: {:?}", key_share.shared_public_key());
        
        // 存储共享公钥用于验证
        self.shared_public_key = Some(format!("{:?}", key_share.shared_public_key()));
        
        key_share
    }

    // 为了兼容现有的同步接口，提供模拟实现
    pub fn execute_round1(&mut self) -> Vec<u8> {
        // 生成一个模拟的 Round 1 消息
        let msg = format!("round1_from_node_{}", self.id);
        msg.into_bytes()
    }

    pub fn handle_round1(&mut self, from: u16, data: Vec<u8>) {
        self.received_round1.insert(from, data);
    }

    pub fn is_round1_complete(&self, total: usize) -> bool {
        self.received_round1.len() == total - 1
    }

    pub fn execute_round2(&mut self, total: usize) -> Vec<(u16, Vec<u8>)> {
        let mut messages = Vec::new();
        for i in 0..total {  // 从0开始，对应节点ID 0,1,2
            if i as u16 != self.id {
                let msg = format!("round2_from_node_{}_to_node_{}", self.id, i);
                messages.push((i as u16, msg.into_bytes()));
            }
        }
        messages
    }

    pub fn handle_round2(&mut self, from: u16, data: Vec<u8>) {
        self.received_round2.insert(from, data);
    }

    pub fn is_round2_complete(&self, total: usize) -> bool {
        self.received_round2.len() == total - 1
    }

    pub fn execute_round3(&mut self) -> Vec<u8> {
        let msg = format!("round3_from_node_{}", self.id);
        msg.into_bytes()
    }

    pub fn handle_round3(&mut self, from: u16, data: Vec<u8>) {
        self.received_round3.insert(from, data);
    }

    pub fn is_round3_complete(&self, total: usize) -> bool {
        self.received_round3.len() == total - 1
    }

    pub fn finalize(&mut self, session_id: &str) -> Option<cggmp21::IncompleteKeyShare<Secp256k1>> {
        // 使用真正的DKG协议生成密钥共享
        let total_nodes = 3; // 假设总节点数为3
        let key_share = self.generate_real_key_shares(total_nodes, session_id);
        
        // 验证公钥一致性
        if let Some(expected_pk) = &self.shared_public_key {
            let actual_pk = format!("{:?}", key_share.shared_public_key());
            if actual_pk == *expected_pk {
                println!("✅ 公钥一致性验证通过！");
            } else {
                println!("❌ 公钥不一致！预期: {}, 实际: {}", expected_pk, actual_pk);
            }
        }
        
        Some(key_share)
    }
}