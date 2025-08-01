# MPC Server-Client 架构图

## 🔒 系统架构与隐私保护

### 1. 整体架构图

```mermaid
graph TB
    subgraph "MPC 参与方"
        P0[服务端 P0<br/>协调器 + MPC参与方]
        P1[客户端 P1<br/>MPC参与方]
        P2[客户端 P2<br/>MPC参与方]
    end
    
    subgraph "HTTP 通信层"
        API[HTTP API<br/>RESTful 接口]
        MSG[消息传递<br/>JSON 格式]
    end
    
    subgraph "本地存储"
        S0[P0 本地存储<br/>私钥份额]
        S1[P1 本地存储<br/>私钥份额]
        S2[P2 本地存储<br/>私钥份额]
    end
    
    subgraph "CGGMP21 协议"
        DKG[DKG 协议<br/>分布式密钥生成]
        SIG[阈值签名<br/>2/3 阈值]
    end
    
    P0 --> API
    P1 --> API
    P2 --> API
    
    P0 --> S0
    P1 --> S1
    P2 --> S2
    
    API --> MSG
    MSG --> DKG
    DKG --> SIG
    
    style P0 fill:#ff9999
    style P1 fill:#99ccff
    style P2 fill:#99ff99
    style S0 fill:#ffcccc
    style S1 fill:#cce5ff
    style S2 fill:#ccffcc
```

### 2. 隐私保护机制图

```mermaid
graph LR
    subgraph "客户端 P1"
        P1_Private[私钥份额<br/>本地存储]
        P1_Public[公钥信息<br/>可共享]
    end
    
    subgraph "客户端 P2"
        P2_Private[私钥份额<br/>本地存储]
        P2_Public[公钥信息<br/>可共享]
    end
    
    subgraph "服务端 P0"
        P0_Private[私钥份额<br/>本地存储]
        P0_Public[公钥信息<br/>可共享]
        Server_Coord[协调服务<br/>会话管理]
    end
    
    subgraph "服务器可见信息"
        Session[会话元数据<br/>参与方列表]
        Shared_PK[共享公钥<br/>027a0d193affe...]
        Messages[消息内容<br/>hello world]
    end
    
    subgraph "服务器不可见信息"
        Private_Keys[私钥份额<br/>❌ 完全不可见]
        Random_Seeds[随机数种子<br/>❌ 完全不可见]
        Signing_Keys[签名私钥<br/>❌ 完全不可见]
    end
    
    P1_Private -.->|❌ 不可访问| Server_Coord
    P2_Private -.->|❌ 不可访问| Server_Coord
    P0_Private -.->|❌ 不可访问| Server_Coord
    
    P1_Public --> Shared_PK
    P2_Public --> Shared_PK
    P0_Public --> Shared_PK
    
    style Private_Keys fill:#ffcccc
    style Random_Seeds fill:#ffcccc
    style Signing_Keys fill:#ffcccc
    style Session fill:#ccffcc
    style Shared_PK fill:#ccffcc
    style Messages fill:#ccffcc
```

### 3. DKG 协议流程图

```mermaid
sequenceDiagram
    participant P0 as 服务端 P0
    participant P1 as 客户端 P1
    participant P2 as 客户端 P2
    participant Server as HTTP 服务器
    
    Note over P0,P2: 分布式密钥生成 (DKG) 协议
    
    P0->>Server: POST /api/mpc/dkg (会话信息)
    P1->>Server: POST /api/mpc/dkg (会话信息)
    P2->>Server: POST /api/mpc/dkg (会话信息)
    
    Note over P0,P2: 每个参与方独立生成私钥份额
    
    P0->>P0: 生成私钥份额 (本地)
    P1->>P1: 生成私钥份额 (本地)
    P2->>P2: 生成私钥份额 (本地)
    
    Note over P0,P2: 所有参与方生成相同的共享公钥
    
    P0->>P0: 保存私钥份额到本地存储
    P1->>P1: 保存私钥份额到本地存储
    P2->>P2: 保存私钥份额到本地存储
    
    Note over P0,P2: 服务器只能看到共享公钥，无法访问私钥份额
    
    P0->>Server: 共享公钥: 027a0d193affe...
    P1->>Server: 共享公钥: 027a0d193affe...
    P2->>Server: 共享公钥: 027a0d193affe...
```

### 4. 阈值签名流程图

```mermaid
sequenceDiagram
    participant P0 as 服务端 P0
    participant P1 as 客户端 P1
    participant P2 as 客户端 P2
    participant Server as HTTP 服务器
    
    Note over P0,P2: 阈值签名协议 (2/3)
    
    P0->>Server: POST /api/mpc/sign (消息: "hello world")
    P1->>Server: POST /api/mpc/sign (消息: "hello world")
    
    Note over P0,P2: 每个参与方使用本地私钥份额生成部分签名
    
    P0->>P0: 使用本地私钥份额生成部分签名
    P1->>P1: 使用本地私钥份额生成部分签名
    
    Note over P0,P2: 服务器无法看到私钥份额，只能看到部分签名
    
    P0->>Server: 部分签名 (不包含私钥)
    P1->>Server: 部分签名 (不包含私钥)
    
    Note over P0,P2: 需要至少2个参与方的部分签名才能生成完整签名
    
    Server->>Server: 组合部分签名生成完整签名
```

### 5. 数据流隐私保护图

```mermaid
graph TD
    subgraph "客户端本地 (私密区域)"
        Private_Share[私钥份额<br/>本地存储]
        Local_RNG[随机数生成器<br/>本地种子]
        Signing_Key[签名私钥<br/>本地计算]
    end
    
    subgraph "HTTP 通信 (公开区域)"
        Session_Info[会话信息<br/>参与方列表]
        Shared_PK[共享公钥<br/>027a0d193affe...]
        Message[消息内容<br/>hello world]
        Partial_Sig[部分签名<br/>不包含私钥]
    end
    
    subgraph "服务器可见 (公开区域)"
        Server_Session[会话管理<br/>元数据]
        Server_PK[公钥存储<br/>共享信息]
        Server_Coord[协议协调<br/>消息转发]
    end
    
    Private_Share -.->|❌ 永不传输| HTTP_Comm
    Local_RNG -.->|❌ 永不传输| HTTP_Comm
    Signing_Key -.->|❌ 永不传输| HTTP_Comm
    
    Session_Info --> Server_Session
    Shared_PK --> Server_PK
    Message --> Server_Coord
    Partial_Sig --> Server_Coord
    
    style Private_Share fill:#ffcccc
    style Local_RNG fill:#ffcccc
    style Signing_Key fill:#ffcccc
    style Session_Info fill:#ccffcc
    style Shared_PK fill:#ccffcc
    style Message fill:#ccffcc
    style Partial_Sig fill:#ccffcc
```

### 6. 安全边界图

```mermaid
graph TB
    subgraph "安全边界 1: 客户端本地"
        C1[客户端 P1<br/>私钥份额存储]
        C2[客户端 P2<br/>私钥份额存储]
        C0[服务端 P0<br/>私钥份额存储]
    end
    
    subgraph "安全边界 2: HTTP 通信"
        API[HTTP API<br/>公开信息传输]
        MSG[消息传递<br/>JSON 格式]
    end
    
    subgraph "安全边界 3: 服务器协调"
        COORD[协议协调<br/>会话管理]
        STORAGE[元数据存储<br/>不包含私钥]
    end
    
    subgraph "安全边界 4: CGGMP21 协议"
        DKG_PROTO[分布式密钥生成<br/>阈值分割]
        SIG_PROTO[阈值签名<br/>协作计算]
    end
    
    C1 -.->|❌ 私钥不传输| API
    C2 -.->|❌ 私钥不传输| API
    C0 -.->|❌ 私钥不传输| API
    
    API --> MSG
    MSG --> COORD
    COORD --> STORAGE
    
    DKG_PROTO --> C1
    DKG_PROTO --> C2
    DKG_PROTO --> C0
    
    SIG_PROTO --> C1
    SIG_PROTO --> C2
    SIG_PROTO --> C0
    
    style C1 fill:#ffcccc
    style C2 fill:#ffcccc
    style C0 fill:#ffcccc
    style API fill:#ccffcc
    style MSG fill:#ccffcc
    style COORD fill:#ccffcc
    style STORAGE fill:#ccffcc
```

## 🔒 隐私保护总结

### ✅ **服务器无法获取的信息**：
- **私钥份额** - 每个客户端的私钥份额只存储在本地
- **随机数种子** - 每个参与方的随机数生成器是独立的
- **签名私钥** - 签名过程中私钥始终在本地
- **完整私钥** - 任何单个参与方都无法获得完整私钥

### ✅ **服务器只能看到的信息**：
- **会话元数据** - 会话ID、参与方列表、阈值等
- **共享公钥** - 所有参与方都知道的公钥
- **消息内容** - 要签名的消息
- **部分签名** - 已经生成的部分签名

### 🎯 **隐私保护机制**：
1. **本地存储** - 所有私钥信息只存储在客户端本地
2. **阈值分割** - 私钥被分割成多个份额，单个份额无意义
3. **协作签名** - 需要多个参与方合作才能生成有效签名
4. **协议隔离** - 服务器只负责协调，不参与私钥操作 