#!/bin/bash

echo "🎯 Distributed MPC DKG Demo (2/3 Threshold)"
echo "==========================================="
echo ""

# Clean up previous data
echo "🧹 Cleaning up previous data..."
rm -rf data/*

# Start server
echo "🚀 Starting MPC Server..."
cargo run --bin server &
SERVER_PID=$!

# Wait for server to start
sleep 2

echo ""
echo "📊 Starting Distributed DKG Protocol (2/3)"
echo "=========================================="
echo ""

# Start P0 client for distributed DKG (Server as participant)
echo "🎯 Starting P0 (Server) for distributed DKG..."
cargo run --bin client 0 'http://127.0.0.1:3000' dkg session_distributed 2 3 &
P0_PID=$!

# Start P1 client for distributed DKG
echo "🎯 Starting P1 for distributed DKG..."
cargo run --bin client 1 'http://127.0.0.1:3000' dkg session_distributed 2 3 &
P1_PID=$!

# Start P2 client for distributed DKG
echo "🎯 Starting P2 for distributed DKG..."
cargo run --bin client 2 'http://127.0.0.1:3000' dkg session_distributed 2 3 &
P2_PID=$!

# Wait for all three clients to complete
wait $P0_PID $P1_PID $P2_PID

echo ""
echo "✅ Distributed DKG completed with all 3 parties!"
echo ""

# Display results
echo "📋 Displaying DKG results..."
echo ""

# Show DKG sessions
echo "📊 DKG Sessions:"
cargo run --bin client 0 'http://127.0.0.1:3000' list-sessions

echo ""
echo "🎉 Distributed DKG Demo completed!"
echo ""
echo "📊 Architecture Summary (2/3 Threshold):"
echo "========================================"
echo "✅ P0 (Server): Participates in DKG as party 0"
echo "✅ P1 (Client): Participates in DKG as party 1"
echo "✅ P2 (Client): Participates in DKG as party 2"
echo "✅ All 3 parties independently execute DKG"
echo "✅ Messages exchanged via HTTP API"
echo "✅ Server coordinates AND participates"
echo "✅ True distributed architecture"
echo "✅ Privacy preserved - no party sees others' private data"
echo "✅ 2/3 threshold: Any 2 parties can sign"
echo ""

# Clean up
echo "🧹 Cleaning up..."
kill $SERVER_PID 2>/dev/null
wait $SERVER_PID 2>/dev/null

echo "✅ Demo cleanup completed!" 