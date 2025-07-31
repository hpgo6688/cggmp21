#!/bin/bash

# Simple MPC Test Script
# This script tests the basic MPC functionality

set -e

echo "=== Simple MPC Test ==="
echo "Testing 2/3 threshold MPC system"
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust first."
    exit 1
fi

# Build the project
print_status "Building MPC server client..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

print_success "Build completed successfully!"

# Create data directories
print_status "Creating data directories..."
mkdir -p data/server
mkdir -p data/client_0
mkdir -p data/client_1
mkdir -p data/client_2

# Test basic functionality
print_status "Testing basic functionality..."
cargo run --release --bin test_basic

if [ $? -eq 0 ]; then
    print_success "Basic functionality test passed!"
else
    echo "❌ Basic functionality test failed!"
    exit 1
fi

# Test DKG simulation
print_status "Testing DKG simulation..."
cargo run --release --bin client 0 http://127.0.0.1:3000 dkg test_session 2 3

if [ $? -eq 0 ]; then
    print_success "DKG simulation test passed!"
else
    echo "❌ DKG simulation test failed!"
    exit 1
fi

# Test signing simulation
print_status "Testing signing simulation..."
cargo run --release --bin client 0 http://127.0.0.1:3000 sign test_session "hello world" 0 1

if [ $? -eq 0 ]; then
    print_success "Signing simulation test passed!"
else
    echo "❌ Signing simulation test failed!"
    exit 1
fi

# Test key share listing
print_status "Testing key share listing..."
cargo run --release --bin client 0 http://127.0.0.1:3000 list-shares test_session

if [ $? -eq 0 ]; then
    print_success "Key share listing test passed!"
else
    echo "❌ Key share listing test failed!"
    exit 1
fi

# Test private key reconstruction
print_status "Testing private key reconstruction..."
cargo run --release --bin client 0 http://127.0.0.1:3000 reconstruct test_session

if [ $? -eq 0 ]; then
    print_success "Private key reconstruction test passed!"
else
    echo "❌ Private key reconstruction test failed!"
    exit 1
fi

print_success "All tests completed successfully!"
echo ""
echo "=== Test Summary ==="
echo "✅ Basic functionality test"
echo "✅ DKG simulation test"
echo "✅ Signing simulation test"
echo "✅ Key share listing test"
echo "✅ Private key reconstruction test"
echo ""
echo "The MPC system is working correctly!" 