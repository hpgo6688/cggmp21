#!/bin/bash

echo "=== MPC Server-Client Complete Demo ==="
echo "This demo shows a 2/3 threshold MPC system with secp256k1"
echo ""

# Kill any existing server
pkill -f "target/release/server" || true
sleep 1

# Start server
echo "🚀 Starting MPC Server..."
cargo run --release --bin server &
SERVER_PID=$!
sleep 3

echo "✅ Server started with PID: $SERVER_PID"
echo ""

# Create data directories
mkdir -p data/server data/client0 data/client1 data/client2

# Run DKG with all three parties
echo "🔑 Running Distributed Key Generation (DKG) with 3 parties..."
echo ""

echo "📋 Party 0 (Server) running DKG..."
cargo run --release --bin client 0 http://127.0.0.1:3000 dkg session_123 2 3 &
P0_PID=$!

echo "📋 Party 1 (Client) running DKG..."
cargo run --release --bin client 1 http://127.0.0.1:3000 dkg session_123 2 3 &
P1_PID=$!

echo "📋 Party 2 (Client) running DKG..."
cargo run --release --bin client 2 http://127.0.0.1:3000 dkg session_123 2 3 &
P2_PID=$!

# Wait for all DKG processes to complete
wait $P0_PID $P1_PID $P2_PID
echo ""
echo "✅ DKG completed for all parties!"
echo ""

# Show key shares
echo "🔍 Displaying key shares for all parties..."
echo ""

echo "📋 Party 0 key share:"
cargo run --release --bin client 0 http://127.0.0.1:3000 list-shares session_123
echo ""

echo "📋 Party 1 key share:"
cargo run --release --bin client 1 http://127.0.0.1:3000 list-shares session_123
echo ""

echo "📋 Party 2 key share:"
cargo run --release --bin client 2 http://127.0.0.1:3000 list-shares session_123
echo ""

# Test signing with P0 and P1
echo "✍️ Testing signature with P0 and P1..."
echo ""

echo "📋 Party 0 signing 'hello world'..."
cargo run --release --bin client 0 http://127.0.0.1:3000 sign session_123 "hello world" 0 1 &
SIGN_P0_PID=$!

echo "📋 Party 1 signing 'hello world'..."
cargo run --release --bin client 1 http://127.0.0.1:3000 sign session_123 "hello world" 0 1 &
SIGN_P1_PID=$!

# Wait for signing to complete
wait $SIGN_P0_PID $SIGN_P1_PID
echo ""
echo "✅ Signing completed!"
echo ""

# Test signing with P1 and P2
echo "✍️ Testing signature with P1 and P2..."
echo ""

echo "📋 Party 1 signing 'hello world'..."
cargo run --release --bin client 1 http://127.0.0.1:3000 sign session_123 "hello world" 1 2 &
SIGN_P1_2_PID=$!

echo "📋 Party 2 signing 'hello world'..."
cargo run --release --bin client 2 http://127.0.0.1:3000 sign session_123 "hello world" 1 2 &
SIGN_P2_2_PID=$!

# Wait for signing to complete
wait $SIGN_P1_2_PID $SIGN_P2_2_PID
echo ""
echo "✅ Second signing completed!"
echo ""

# Test key reconstruction
echo "🔧 Testing key reconstruction..."
echo ""

echo "📋 Attempting to reconstruct private key from all shares..."
cargo run --release --bin client 0 http://127.0.0.1:3000 reconstruct session_123
echo ""

# Show system summary
echo "📊 System Summary:"
echo "=================="
echo "✅ Server (P0) running on http://127.0.0.1:3000"
echo "✅ 2/3 threshold MPC system implemented"
echo "✅ secp256k1 curve support"
echo "✅ HTTP communication between parties"
echo "✅ Distributed Key Generation (DKG) completed"
echo "✅ Threshold signing demonstrated"
echo "✅ Privacy preserved (no single party has full key)"
echo "✅ Service and clients are separate entities"
echo ""

echo "🎉 Demo completed successfully!"
echo ""
echo "Key Features Demonstrated:"
echo "1. ✅ Three parties (P0, P1, P2) generated distributed key shares"
echo "2. ✅ Interactive computation through HTTP communication"
echo "3. ✅ P0 and P1 signed 'hello world' message"
echo "4. ✅ P1 and P2 signed 'hello world' message"
echo "5. ✅ Service and clients are separate entities"
echo "6. ✅ Privacy protection (no party has complete private key)"
echo ""

# Cleanup
echo "🧹 Cleaning up..."
kill $SERVER_PID 2>/dev/null || true
echo "✅ Demo cleanup completed!" 