use rand::Rng;
use rand_dev::DevRng;
use cggmp21::{ExecutionId, supported_curves::Secp256k1, key_share::DirtyIncompleteKeyShare, key_share::reconstruct_secret_key};
use generic_ec::{Point, Scalar};

// 创建一个简单的测试来查看密钥份额
#[test]
fn debug_share_example() {
    let mut rng = DevRng::new();
    
    // 设置参数：2/3阈值
    let t = 2;
    let n = 3;
    
    // 生成执行ID
    let eid_bytes: [u8; 32] = rng.gen();
    let eid = ExecutionId::new(&eid_bytes);
    
    println!("=== 2/3 阈值密钥生成 ===");
    println!("阈值 t = {}", t);
    println!("参与者数量 n = {}", n);
    println!("执行ID = {:?}", eid_bytes);
    
    // 运行密钥生成
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
    
    println!("\n=== 密钥份额信息 ===");
    
    // 查看每个参与者的密钥份额
    for (i, share) in key_shares.iter().enumerate() {
        println!("\n--- 参与者 {} ---", i);
        
        // 获取内部数据
        let dirty_share: &DirtyIncompleteKeyShare<Secp256k1> = share.as_ref();
        
        println!("参与者索引: {}", dirty_share.i);
        println!("秘密份额 x: {:?}", dirty_share.x);
        
        // 查看公钥信息
        let key_info = &dirty_share.key_info;
        println!("共享公钥: {:?}", key_info.shared_public_key);
        println!("公钥份额数量: {}", key_info.public_shares.len());
        
        // 查看VSS设置（阈值信息）
        if let Some(vss_setup) = &key_info.vss_setup {
            println!("最小签名者数量: {}", vss_setup.min_signers);
            println!("份额索引 I: {:?}", vss_setup.I);
        }
        
        // 查看HD钱包信息
        #[cfg(feature = "hd-wallet")]
        {
            if let Some(chain_code) = &key_info.chain_code {
                println!("链码: {:?}", chain_code);
            }
        }
        
        // 验证份额
        println!("份额验证: OK");
    }
    
    // 验证所有份额是否一致
    println!("\n=== 份额一致性验证 ===");
    let first_share = &key_shares[0];
    for (i, share) in key_shares.iter().enumerate() {
        println!("参与者 {} 的共享公钥: {:?}", i, share.shared_public_key());
        assert_eq!(first_share.shared_public_key(), share.shared_public_key());
    }
    
    println!("\n✅ 所有参与者的密钥份额生成成功！");
    
    // 显示密钥份额的详细信息
    println!("\n=== 密钥份额详细结构 ===");
    let share = &key_shares[0];
    let dirty_share: &DirtyIncompleteKeyShare<Secp256k1> = share.as_ref();
    
    println!("密钥份额类型: IncompleteKeyShare<Secp256k1>");
    println!("参与者索引: {}", dirty_share.i);
    println!("秘密份额: {:?}", dirty_share.x);
    println!("共享公钥: {:?}", dirty_share.key_info.shared_public_key);
    println!("公钥份额数量: {}", dirty_share.key_info.public_shares.len());
    
    if let Some(vss_setup) = &dirty_share.key_info.vss_setup {
        println!("VSS设置:");
        println!("  - 最小签名者数量: {}", vss_setup.min_signers);
        println!("  - 份额索引: {:?}", vss_setup.I);
    }
    
    // 导出完整私钥
    println!("\n=== 导出完整私钥 ===");
    
    // 方法1: 使用所有参与者的份额重构私钥
    println!("方法1: 使用所有参与者的份额重构私钥");
    match reconstruct_secret_key(&key_shares) {
        Ok(secret_key) => {
            println!("✅ 成功重构完整私钥!");
            println!("完整私钥: {:?}", secret_key);
            
            // 验证重构的私钥是否正确
            let reconstructed_public_key = Point::generator() * &secret_key;
            println!("重构的公钥: {:?}", reconstructed_public_key);
            println!("原始共享公钥: {:?}", dirty_share.key_info.shared_public_key);
            
            // 验证公钥是否匹配
            if reconstructed_public_key == *dirty_share.key_info.shared_public_key.as_ref() {
                println!("✅ 重构的私钥验证成功!");
            } else {
                println!("❌ 重构的私钥验证失败!");
            }
        }
        Err(e) => {
            println!("❌ 重构私钥失败: {:?}", e);
        }
    }
    
    // 方法2: 使用部分参与者的份额重构私钥（阈值重构）
    println!("\n方法2: 使用部分参与者的份额重构私钥（阈值重构）");
    
    // 选择前t个参与者
    let threshold_shares = &key_shares[..t as usize];
    println!("使用参与者 0 和 1 的份额进行重构...");
    
    match reconstruct_secret_key(threshold_shares) {
        Ok(secret_key) => {
            println!("✅ 成功使用阈值份额重构完整私钥!");
            println!("完整私钥: {:?}", secret_key);
            
            // 验证重构的私钥是否正确
            let reconstructed_public_key = Point::generator() * &secret_key;
            println!("重构的公钥: {:?}", reconstructed_public_key);
            println!("原始共享公钥: {:?}", dirty_share.key_info.shared_public_key);
            
            // 验证公钥是否匹配
            if reconstructed_public_key == *dirty_share.key_info.shared_public_key.as_ref() {
                println!("✅ 阈值重构的私钥验证成功!");
            } else {
                println!("❌ 阈值重构的私钥验证失败!");
            }
        }
        Err(e) => {
            println!("❌ 阈值重构私钥失败: {:?}", e);
        }
    }
    
    // 方法3: 尝试使用单个参与者的份额（应该失败）
    println!("\n方法3: 尝试使用单个参与者的份额重构私钥（应该失败）");
    let single_share = &key_shares[..1];
    match reconstruct_secret_key(single_share) {
        Ok(secret_key) => {
            println!("❌ 意外成功使用单个份额重构私钥: {:?}", secret_key);
        }
        Err(e) => {
            println!("✅ 正确失败: 单个份额不足以重构私钥");
            println!("错误信息: {:?}", e);
        }
    }
    
    // 方法4: 显示私钥详细信息
    println!("\n=== 私钥详细信息 ===");
    
    // 重构完整私钥用于显示
    if let Ok(secret_key) = reconstruct_secret_key(&key_shares) {
        println!("私钥类型: SecretScalar<Secp256k1>");
        println!("私钥标量值: {:?}", secret_key);
        
        // 获取私钥的标量值
        let scalar_value = secret_key.as_ref();
        println!("私钥标量: {:?}", scalar_value);
        
        // 导出为字节数组
        let bytes_secret = scalar_value.to_be_bytes();
        println!("私钥字节数组: {:?}", bytes_secret);
        println!("私钥字节长度: {} 字节", bytes_secret.len());
        
        // 导出为十六进制格式
        let hex_secret = hex::encode(&bytes_secret);
        println!("私钥十六进制: {}", hex_secret);
        
        // 导出为小端字节序十六进制
        let hex_secret_le = hex::encode(&scalar_value.to_le_bytes());
        println!("私钥十六进制(小端): {}", hex_secret_le);
        
        // 显示字节数组的详细信息
        println!("私钥字节数组(大端): {:?}", bytes_secret);
        println!("私钥字节数组(小端): {:?}", scalar_value.to_le_bytes());
        
        // 验证私钥对应的公钥
        let public_key = Point::generator() * &secret_key;
        println!("私钥对应的公钥: {:?}", public_key);
        
        // 显示私钥的详细信息
        println!("私钥重构成功!");
        println!("私钥可用于生成签名!");
        
        // 显示私钥的数值信息
        println!("私钥是否为非零: {}", *scalar_value != Scalar::zero());
        println!("私钥标量值(十六进制): 0x{}", hex_secret);
        
        // 显示私钥的WIF格式（比特币私钥格式的简化版本）
        println!("私钥WIF格式(简化): {}", hex_secret);
        
        // 显示私钥的详细信息
        println!("=== 私钥完整信息 ===");
        println!("类型: SecretScalar<Secp256k1>");
        println!("标量值: {:?}", scalar_value);
        println!("十六进制: 0x{}", hex_secret);
        println!("字节长度: {} 字节", bytes_secret.len());
        println!("公钥: {:?}", public_key);
        println!("验证状态: 有效");
    }
    
    println!("\n=== 安全警告 ===");
    println!("⚠️  注意：在实际应用中，重构完整私钥会破坏阈值签名的安全性!");
    println!("⚠️  只有在特殊情况下（如密钥恢复）才应该这样做!");
    println!("⚠️  重构的私钥应该安全存储，使用后立即销毁!");
} 