#!/bin/bash

# 启动中继服务器
echo "Starting relay server..."
cd relay-server
cargo run &
RELAY_PID=$!
cd ..

# 等待中继服务器启动
sleep 3

# 启动 DKG 节点
echo "Starting DKG nodes..."
cd dkg-node

# 启动第一个会话的节点
echo "Starting session-a nodes..."
cargo run -- --id 0 --relay http://127.0.0.1:9000 --total 3 --session session-a &
NODE1_PID=$!

cargo run -- --id 1 --relay http://127.0.0.1:9000 --total 3 --session session-a &
NODE2_PID=$!

cargo run -- --id 2 --relay http://127.0.0.1:9000 --total 3 --session session-a &
NODE3_PID=$!

# 等待第一个会话完成
sleep 10

# 启动第二个会话的节点（并行测试）
echo " "
echo " "
echo " "
echo " "
echo "Starting session-b nodes..."
cargo run -- --id 0 --relay http://127.0.0.1:9000 --total 3 --session session-b &
NODE4_PID=$!

cargo run -- --id 1 --relay http://127.0.0.1:9000 --total 3 --session session-b &
NODE5_PID=$!

cargo run -- --id 2 --relay http://127.0.0.1:9000 --total 3 --session session-b &
NODE6_PID=$!

# 等待所有进程完成
wait $NODE1_PID $NODE2_PID $NODE3_PID $NODE4_PID $NODE5_PID $NODE6_PID
# wait $NODE1_PID $NODE2_PID $NODE3_PID

# 清理
kill $RELAY_PID
cd ..

echo "Test completed!" 