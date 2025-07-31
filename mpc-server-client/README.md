# MPC Server Client

A complete implementation of MPC (Multi-Party Computation) server and client for distributed key generation (DKG) and threshold signing using the CGGMP21 protocol with secp256k1 curve.

## Features

- **2/3 Threshold MPC**: Supports threshold signing where any 2 out of 3 parties can sign
- **Distributed Key Generation (DKG)**: Secure generation of distributed key shares
- **Threshold Signing**: Sign messages with any subset of parties meeting the threshold
- **HTTP Communication**: All parties communicate via HTTP REST API
- **Persistent Storage**: Key shares and session data are stored on disk
- **secp256k1 Support**: Uses the secp256k1 curve for compatibility with Bitcoin/Ethereum

## Architecture

```
┌─────────────┐    HTTP    ┌─────────────┐    HTTP    ┌─────────────┐
│   Party 0   │◄──────────►│   Server    │◄──────────►│   Party 1   │
│  (Server)   │             │             │             │  (Client)   │
└─────────────┘             └─────────────┘             └─────────────┘
                                    │
                                    │ HTTP
                                    ▼
                            ┌─────────────┐
                            │   Party 2   │
                            │  (Client)   │
                            └─────────────┘
```

- **Party 0**: Acts as both server and participant
- **Party 1 & 2**: Client participants
- **Server**: Central message relay and session management
- **HTTP API**: RESTful communication between all parties

## Installation

1. Ensure you have Rust installed (1.70+)
2. Clone this repository
3. Build the project:

```bash
cargo build --release
```

## Usage

### Starting the Server

```bash
cargo run --release --bin server
```

The server will start on `http://127.0.0.1:3000` and provide the following API endpoints:

- `POST /api/mpc/dkg` - Start DKG protocol
- `POST /api/mpc/sign` - Start signing protocol
- `GET /api/mpc/sessions` - List all sessions
- `GET /api/mpc/key-shares/:session_id` - Get key shares
- `POST /api/mpc/message` - Receive protocol message
- `GET /api/mpc/messages/:session_id/:round` - Get round messages

### Running Clients

The client supports several commands:

#### DKG (Distributed Key Generation)

```bash
# Party 1
cargo run --release --bin client 1 http://127.0.0.1:3000 dkg session_123 2 3

# Party 2  
cargo run --release --bin client 2 http://127.0.0.1:3000 dkg session_123 2 3
```

#### Signing

```bash
# Sign with Party 0 and Party 1
cargo run --release --bin client 0 http://127.0.0.1:3000 sign session_123 "hello world" 0 1
cargo run --release --bin client 1 http://127.0.0.1:3000 sign session_123 "hello world" 0 1

# Sign with Party 1 and Party 2
cargo run --release --bin client 1 http://127.0.0.1:3000 sign session_123 "hello world" 1 2
cargo run --release --bin client 2 http://127.0.0.1:3000 sign session_123 "hello world" 1 2
```

#### Key Management

```bash
# List key shares
cargo run --release --bin client 0 http://127.0.0.1:3000 list-shares session_123

# Reconstruct private key
cargo run --release --bin client 0 http://127.0.0.1:3000 reconstruct session_123

# Verify signature
cargo run --release --bin client 0 http://127.0.0.1:3000 verify session_123 "hello world" <r_hex> <s_hex>
```

## Demo

Run the complete demo script to see the system in action:

```bash
chmod +x test_demo.sh
./test_demo.sh
```

This demo will:

1. Start the MPC server
2. Run DKG with all three parties (2/3 threshold)
3. Test signing with P0+P1 combination
4. Test signing with P1+P2 combination
5. Verify key consistency across all combinations
6. Demonstrate private key reconstruction

## Protocol Flow

### DKG (Distributed Key Generation)

1. All parties join the DKG session
2. Each party generates their share of the private key
3. Parties exchange messages to establish the distributed key
4. Each party receives their key share
5. The public key is derived from all shares

### Threshold Signing

1. Participating parties join the signing session
2. Parties exchange messages to generate the signature
3. Each party contributes their share to the final signature
4. The signature is verified against the public key

## Security Features

- **Threshold Security**: No single party can sign alone
- **Distributed Trust**: No single point of failure
- **Cryptographic Verification**: All signatures are cryptographically verified
- **Session Isolation**: Each session has unique execution ID
- **Persistent Storage**: Key shares are securely stored

## Data Storage

The system stores data in the following structure:

```
data/
├── server/
│   ├── sessions/
│   ├── key_shares/
│   ├── presignatures/
│   └── partial_signatures/
├── client_0/
├── client_1/
└── client_2/
```

## API Reference

### DKG Request

```json
{
  "session_id": {
    "id": "session_123",
    "execution_id": [0, 0, ...]
  },
  "party_id": 1,
  "party_address": "http://localhost:3001",
  "threshold": 2,
  "total_parties": 3
}
```

### Sign Request

```json
{
  "session_id": {
    "id": "session_123",
    "execution_id": [0, 0, ...]
  },
  "party_id": 1,
  "message": "hello world",
  "participating_parties": [0, 1]
}
```

### API Response

```json
{
  "success": true,
  "data": {...},
  "error": null
}
```

## Testing

The system includes comprehensive tests:

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_mpc_server_client_2_3_threshold
```

## Configuration

Key configuration options:

- **Threshold**: Set to 2 for 2/3 threshold
- **Total Parties**: Set to 3 for three-party system
- **Curve**: secp256k1 for Bitcoin/Ethereum compatibility
- **Server Port**: Default 3000, configurable
- **Storage Path**: Default `data/`, configurable

## Troubleshooting

### Common Issues

1. **Server won't start**: Check if port 3000 is available
2. **DKG fails**: Ensure all parties are running and can reach the server
3. **Signing fails**: Verify that participating parties have valid key shares
4. **Storage errors**: Check file permissions for data directories

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=debug cargo run --release --bin server
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Acknowledgments

This implementation is based on the CGGMP21 paper and uses the cggmp21 Rust library for the core MPC protocols. 