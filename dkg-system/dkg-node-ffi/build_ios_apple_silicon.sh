#!/bin/bash

# Build script for iOS on Apple Silicon Mac
# This script uses the force-cross feature to build iOS libraries

set -e

echo "🍎 Building DKG Node FFI Library for iOS on Apple Silicon..."

# Check if we're on Apple Silicon
PLATFORM=$(uname -m)
if [[ "$PLATFORM" != "arm64" ]]; then
    echo "❌ This script is for Apple Silicon Macs only"
    echo "💻 For Intel Macs, use the regular build.sh script"
    exit 1
fi

echo "📋 Detected platform: $PLATFORM"

# Build for macOS first
echo "📦 Building for macOS..."
cargo build --release

# Build for iOS using force-cross feature
echo "📱 Building for iOS Device (arm64) with force-cross..."
cargo build --release --target aarch64-apple-ios --features gmp-mpfr-sys/force-cross

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
echo ""
echo "⚠️  Note: iOS Simulator (x86_64) is not supported on Apple Silicon"
echo "📖 See README.md for Swift integration instructions" 