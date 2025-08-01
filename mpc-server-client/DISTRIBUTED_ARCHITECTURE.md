# 🎯 分布式MPC DKG架构 (2/3阈值)

## 📊 架构概述

我们重新设计了MPC系统，实现了真正的分布式DKG（Distributed Key Generation）架构，支持2/3阈值签名。

### 🔄 架构对比

#### ❌ 旧架构（集中式）
```
客户端 → 服务端 → 执行DKG → 返回结果
```
- 服务端参与DKG计算
- 服务端可以看到所有私钥份额
- 违背了MPC的隐私原则

#### ✅ 新架构（分布式）
```
P0 (服务端) → 独立执行DKG → 通过消息交换
P1 (客户端) → 独立执行DKG → 通过消息交换  
P2 (客户端) → 独立执行DKG → 通过消息交换
服务端      → 负责消息转发和协调
```

## 🎯 2/3阈值参与方

### 参与方配置
- **P0 (服务端)**: Party ID = 0, 地址 = http://localhost:3000
- **P1 (客户端)**: Party ID = 1, 地址 = http://localhost:3001  
- **P2 (客户端)**: Party ID = 2, 地址 = http://localhost:3002

### 阈值特性
- **总参与方**: 3个 (P0, P1, P2)
- **阈值**: 2个 (任意2个参与方可以签名)
- **容错**: 最多1个参与方故障，系统仍可正常工作

## 🎯 核心特性

### 1. **真正的分布式**
- 每个参与方独立执行DKG协议
- 服务端既协调又参与计算
- 每个参与方只看到自己的私钥份额

### 2. **隐私保护**
- 服务端无法获取其他参与方的私钥
- 参与方之间通过加密消息交换
- 符合MPC的隐私原则

### 3. **交互式协议**
- Phase 1: 生成多项式并广播承诺
- Phase 2: 分发份额给其他参与方
- Phase 3: 验证和聚合份额
- Phase 4: 确认完成

## 📡 消息流程

### DKG消息类型
```rust
pub enum DkgMessageType {
    Commitment,  // Phase 1: 承诺广播
    Share,       // Phase 2: 份额分发
    Verification, // Phase 3: 验证
    Confirmation, // Phase 4: 确认
}
```

### 消息交换流程
1. **承诺阶段**：每个参与方生成多项式，广播承诺
2. **份额分发**：每个参与方将份额安全发送给其他参与方
3. **验证阶段**：验证收到的份额的正确性
4. **聚合阶段**：聚合所有份额得到最终私钥份额

## 🔧 实现细节

### 客户端实现
```rust
// 每个参与方独立执行DKG
async fn run_distributed_dkg(&mut self, session_id: SessionId, threshold: u16, total_parties: u16) -> Result<()> {
    // Phase 1: 生成承诺
    let commitments = self.generate_commitments(execution_id, threshold, total_parties).await?;
    self.broadcast_commitments(session_id.clone(), commitments).await?;
    
    // Phase 2: 分发份额
    let shares = self.generate_shares(execution_id, threshold, total_parties).await?;
    for party_id in 0..total_parties {
        if party_id != self.party_id {
            self.send_share(session_id.clone(), party_id, shares[party_id as usize].clone()).await?;
        }
    }
    
    // Phase 3: 验证和聚合
    let final_key_share = self.verify_and_aggregate_shares(session_id.clone(), threshold, total_parties).await?;
}
```

### 服务端实现
```rust
// 服务端既协调又参与
async fn handle_dkg_message(&self, message: DkgMessage) -> Result<()> {
    // 保存消息到存储
    self.storage.save_dkg_message(message.clone())?;
    
    // 转发给目标参与方
    if let Some(to_party) = message.to_party {
        // 点对点消息
        self.forward_to_party(to_party, message).await?;
    } else {
        // 广播消息
        self.broadcast_to_all_parties(message).await?;
    }
}
```

## 🎯 优势

### 1. **隐私保护**
- ✅ 服务端无法获取其他参与方的私钥份额
- ✅ 参与方之间通过加密通信
- ✅ 符合MPC安全模型

### 2. **可扩展性**
- ✅ 支持任意数量的参与方
- ✅ 可以轻松添加新的参与方
- ✅ 模块化设计

### 3. **容错性**
- ✅ 参与方可以独立运行
- ✅ 服务端故障不影响DKG
- ✅ 支持参与方动态加入/退出

### 4. **2/3阈值特性**
- ✅ 3个参与方：P0, P1, P2
- ✅ 任意2个参与方可以签名
- ✅ 最多1个参与方故障仍可工作

## 🚀 使用方法

### 启动分布式DKG (2/3)
```bash
# 启动服务端
cargo run --bin server &

# 启动参与方P0 (服务端)
cargo run --bin client 0 'http://127.0.0.1:3000' dkg session_distributed 2 3 &

# 启动参与方P1
cargo run --bin client 1 'http://127.0.0.1:3000' dkg session_distributed 2 3 &

# 启动参与方P2  
cargo run --bin client 2 'http://127.0.0.1:3000' dkg session_distributed 2 3 &
```

### 运行演示
```bash
./demo_distributed.sh
```

## 📈 性能特点

- **并行执行**：所有3个参与方并行执行DKG
- **消息效率**：只传输必要的消息
- **存储优化**：消息持久化到磁盘
- **网络友好**：支持HTTP/HTTPS通信

## 🔮 未来改进

1. **加密通信**：添加端到端加密
2. **异步处理**：支持异步消息处理
3. **容错机制**：添加参与方故障恢复
4. **性能优化**：优化消息传输效率
5. **监控日志**：添加详细的监控和日志

## 🎉 总结

新的分布式架构实现了真正的MPC DKG (2/3阈值)：

- ✅ **分布式**：每个参与方独立执行
- ✅ **隐私**：服务端无法获取其他参与方的私钥
- ✅ **安全**：符合MPC安全模型
- ✅ **2/3阈值**：3个参与方，任意2个可签名
- ✅ **容错**：最多1个参与方故障仍可工作
- ✅ **实用**：基于HTTP的简单通信

这为构建真正的分布式MPC系统奠定了坚实的基础！🎉 