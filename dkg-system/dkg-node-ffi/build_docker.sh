#!/bin/bash

# Build script using Docker for cross-compilation on Apple Silicon
# This script builds iOS libraries using Docker to avoid cross-compilation issues

set -e

echo "🐳 Building DKG Node FFI Library for iOS using Docker..."

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    echo "💡 Visit: https://docs.docker.com/get-docker/"
    exit 1
fi

# Check if we're on Apple Silicon
PLATFORM=$(uname -m)
if [[ "$PLATFORM" != "arm64" ]]; then
    echo "❌ This script is optimized for Apple Silicon Macs"
    echo "💻 For Intel Macs, use the regular build.sh script"
    exit 1
fi

echo "📋 Detected platform: $PLATFORM"

# Build for macOS first (native)
echo "📦 Building for macOS (native)..."
cargo build --release

# Build for iOS using Docker
echo "📱 Building for iOS using Docker..."
docker build -f Dockerfile.ios -t dkg-node-ffi-ios .

# Copy built libraries from Docker container
echo "📋 Copying built libraries..."
docker create --name temp-container dkg-node-ffi-ios
docker cp temp-container:/app/target/aarch64-apple-ios/release/libdkg_node_ffi.a target/aarch64-apple-ios/release/ 2>/dev/null || mkdir -p target/aarch64-apple-ios/release && docker cp temp-container:/app/target/aarch64-apple-ios/release/libdkg_node_ffi.a target/aarch64-apple-ios/release/
docker cp temp-container:/app/target/x86_64-apple-ios/release/libdkg_node_ffi.a target/x86_64-apple-ios/release/ 2>/dev/null || mkdir -p target/x86_64-apple-ios/release && docker cp temp-container:/app/target/x86_64-apple-ios/release/libdkg_node_ffi.a target/x86_64-apple-ios/release/
docker rm temp-container

# Show generated files
echo ""
echo "📁 Generated files:"
find target -name "libdkg_node_ffi.*" 2>/dev/null | head -10

echo ""
echo "🎉 iOS build completed successfully!"
echo ""
echo "📋 Usage:"
echo "   - macOS: target/release/libdkg_node_ffi.dylib"
echo "   - iOS Device: target/aarch64-apple-ios/release/libdkg_node_ffi.a"
echo "   - iOS Simulator: target/x86_64-apple-ios/release/libdkg_node_ffi.a"
echo ""
echo "📖 See README.md for Swift integration instructions" 