#!/bin/bash

echo "=== 正确的 MPC 演示 (2/3 阈值) ==="
echo "服务端P0自动参与，只需要调用P1和P2客户端"
echo ""

# Kill any existing server
pkill -f "target/release/server" || true
sleep 1

# Start server (P0自动参与)
echo "🚀 Starting MPC Server (P0自动参与)..."
cargo run --release --bin server &
SERVER_PID=$!
sleep 3

echo "✅ Server started with PID: $SERVER_PID"
echo ""


# 正确的DKG流程：只调用两个客户端，服务端P0自动参与
echo "🔑 Running DKG with 2/3 threshold..."
echo "服务端P0自动参与，只需要调用P1和P2客户端"
echo ""

echo "📋 Party 1 (Client) running DKG..."
cargo run --release --bin client 1 http://127.0.0.1:3000 dkg session_correct 2 3 &
P1_PID=$!

echo "📋 Party 2 (Client) running DKG..."
cargo run --release --bin client 2 http://127.0.0.1:3000 dkg session_correct 2 3 &
P2_PID=$!

# Wait for both client processes to complete
wait $P1_PID $P2_PID
echo ""
echo "✅ DKG completed for P1 and P2! (P0自动参与)"
echo ""

# Show key shares for all parties
echo "🔍 Displaying key shares for all parties..."
echo ""

echo "📋 Party 0 (Server) key share:"
cargo run --release --bin client 0 http://127.0.0.1:3000 list-shares session_correct
echo ""

echo "📋 Party 1 (Client) key share:"
cargo run --release --bin client 1 http://127.0.0.1:3000 list-shares session_correct
echo ""

echo "📋 Party 2 (Client) key share:"
cargo run --release --bin client 2 http://127.0.0.1:3000 list-shares session_correct
echo ""

# Test signing with P0 and P1 (服务端P0自动参与)
echo "✍️ Testing signature with P0 and P1..."
echo ""

echo "📋 Party 1 signing 'hello world'..."
cargo run --release --bin client 1 http://127.0.0.1:3000 sign session_correct "hello world" 0 1 &
SIGN_P1_PID=$!

# Wait for signing to complete
wait $SIGN_P1_PID
echo ""
echo "✅ Signing completed! (P0自动参与)"
echo ""

# Test signing with P1 and P2
echo "✍️ Testing signature with P1 and P2..."
echo ""

echo "📋 Party 1 signing 'hello world'..."
cargo run --release --bin client 1 http://127.0.0.1:3000 sign session_correct "hello world" 1 2 &
SIGN_P1_2_PID=$!

echo "📋 Party 2 signing 'hello world'..."
cargo run --release --bin client 2 http://127.0.0.1:3000 sign session_correct "hello world" 1 2 &
SIGN_P2_2_PID=$!

# Wait for signing to complete
wait $SIGN_P1_2_PID $SIGN_P2_2_PID
echo ""
echo "✅ Second signing completed!"
echo ""

# Show system summary
echo "📊 正确的系统架构:"
echo "=================="
echo "✅ 服务端P0自动参与DKG和签名"
echo "✅ 只需要调用P1和P2客户端"
echo "✅ 2/3阈值MPC系统"
echo "✅ secp256k1曲线支持"
echo "✅ HTTP通信"
echo "✅ 分布式密钥生成"
echo "✅ 阈值签名"
echo "✅ 隐私保护"
echo "✅ 服务端与客户端分离"
echo ""

echo "🎉 正确的演示完成!"
echo ""
echo "关键改进:"
echo "1. ✅ 服务端P0自动参与，不需要客户端调用"
echo "2. ✅ 只需要调用P1和P2两个客户端"
echo "3. ✅ 真正的2/3阈值MPC架构"
echo "4. ✅ 服务端作为协调器+参与方"
echo ""

# Cleanup
echo "🧹 Cleaning up..."
kill $SERVER_PID 2>/dev/null || true
echo "✅ Demo cleanup completed!" 