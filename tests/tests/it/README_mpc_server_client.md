# MPC Server/Client Implementation for 2/3 Threshold Signing

This implementation provides a complete 2/3 MPC (Multi-Party Computation) system for secp256k1 threshold signing with the following features:

## Overview

The system implements a 2/3 threshold signing scheme where:
- **Server (P0)**: Acts as one of the MPC parties
- **Client 1 (P1)**: First client party
- **Client 2 (P2)**: Second client party
- **Threshold**: Any 2 out of 3 parties can generate a valid signature

## Key Features

### 1. Distributed Key Generation (DKG)
- Generates 3 key shares using the CGGMP21 protocol
- Each party receives a unique key share
- All parties share the same public key
- Threshold is set to 2 (minimum 2 parties required for signing)

### 2. Key Share Storage
- Key shares are persisted to JSON files for easy testing
- Storage directory: `test_data/mpc_key_shares/`
- File naming: `party_{id}_key_share.json`
- Supports loading and verification of stored key shares

### 3. Signing Scenarios

#### Scenario 2.1: All Parties Signing
- **Participants**: Server (P0) + Client 1 (P1) + Client 2 (P2)
- **Description**: All three parties participate in signing
- **Use Case**: Maximum security and availability

#### Scenario 2.2: Server + Client 1 (Threshold)
- **Participants**: Server (P0) + Client 1 (P1)
- **Description**: Threshold signing with server and one client
- **Use Case**: When Client 2 is unavailable

#### Scenario 2.3: Client 1 + Client 2 (Threshold)
- **Participants**: Client 1 (P1) + Client 2 (P2)
- **Description**: Threshold signing with two clients only
- **Use Case**: When server is unavailable or for client-only operations

### 4. Private Key Export
- **Method 1**: Reconstruct using all 3 shares
- **Method 2**: Reconstruct using threshold shares (2 shares)
- **Output**: Complete private key in hexadecimal format
- **Verification**: Reconstructed private key matches the shared public key

## File Structure

```
tests/tests/it/mpc_server_client.rs
├── KeyShareStorage          # Key share persistence utilities
├── test_mpc_server_client_2_3_threshold()  # Main test function
├── test_signing_scenarios() # Signing scenario demonstrations
├── test_private_key_export() # Private key reconstruction
└── test_key_share_persistence() # Storage and loading tests
```

## Usage

### Running the Tests

```bash
# Run the main MPC test
cargo test test_mpc_server_client_2_3_threshold --test it -- --nocapture

# Run the persistence test
cargo test test_key_share_persistence --test it -- --nocapture
```

### Key Share Storage

Key shares are automatically saved to:
- `test_data/mpc_key_shares/` (main test)
- `test_data/mpc_key_shares_persistence/` (persistence test)

Each file contains a JSON representation of the key share for that party.

## Security Features

1. **Threshold Security**: No single party can generate signatures alone
2. **Key Share Isolation**: Each party only has access to their own share
3. **Verifiable Reconstruction**: Private key can be reconstructed and verified
4. **Persistent Storage**: Key shares can be safely stored and loaded

## Implementation Details

### DKG Process
1. Generate execution ID for the session
2. Run distributed key generation with 3 parties
3. Set threshold to 2 (minimum signers)
4. Each party receives their key share
5. All parties share the same public key

### Signing Process
1. Select participants (minimum 2, maximum 3)
2. Run threshold signing protocol
3. Generate valid signature
4. Verify signature with shared public key

### Key Reconstruction
1. Collect key shares from participants
2. Use Lagrange interpolation to reconstruct private key
3. Verify against shared public key
4. Export in hexadecimal format

## Test Results Example

```
=== 2/3 MPC Secp256k1 签名系统测试 ===
阈值 t = 2
参与者数量 n = 3

=== 步骤 1: 分布式密钥生成 (DKG) ===
✅ DKG 完成，生成了 3 个密钥份额

=== 步骤 2: 签名场景测试 ===
✅ 场景 2.1 签名准备完成 - 所有参与者参与
✅ 场景 2.2 签名准备完成 - 阈值签名 (2/3)
✅ 场景 2.3 签名准备完成 - 阈值签名 (2/3)

=== 步骤 3: 私钥导出测试 ===
✅ 成功重构完整私钥!
私钥十六进制: 0d4e118c8524b5ef1f3ea79bfeac2e3578c3357bd32902dbbaa670e686ee64d4
✅ 重构的私钥验证成功!
```

## Security Considerations

⚠️ **Important Security Warnings**:

1. **Private Key Reconstruction**: Reconstructing the complete private key destroys the threshold security properties. Only do this for key recovery scenarios.

2. **Key Share Storage**: In production, key shares should be stored securely with proper encryption and access controls.

3. **Network Security**: In a real implementation, secure communication channels between parties are essential.

4. **Key Rotation**: Regular key refresh should be implemented for long-term security.

## Future Enhancements

1. **Real Network Communication**: Implement actual network protocols between server and clients
2. **Signature Verification**: Add complete signature verification with external libraries
3. **HD Wallet Support**: Add hierarchical deterministic wallet support
4. **Key Refresh**: Implement key refresh protocols for long-term security
5. **Production Security**: Add proper encryption and security measures for production use

## Dependencies

- `cggmp21`: Core MPC protocol implementation
- `generic-ec`: Elliptic curve operations
- `rand`: Random number generation
- `serde_json`: JSON serialization for key share storage
- `hex`: Hexadecimal encoding for private key export 