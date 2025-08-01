#!/bin/bash

# Build script for dkg-node-ffi
# This script builds the FFI library for different platforms

set -e

echo "🔨 Building DKG Node FFI Library..."

# Detect platform
PLATFORM=$(uname -m)
echo "📋 Detected platform: $PLATFORM"

# Build for macOS
echo "📦 Building for macOS..."
cargo build --release

# Build for iOS (if available)
if command -v xcrun &> /dev/null; then
    echo "📱 Building for iOS..."
    
    # Check if we're on Apple Silicon
    if [[ "$PLATFORM" == "arm64" ]]; then
        echo "🍎 Apple Silicon detected - skipping iOS builds due to gmp-mpfr-sys cross-compilation issues"
        echo "💡 To build for iOS on Apple Silicon, you may need to:"
        echo "   1. Install the force-cross feature: cargo build --release --target aarch64-apple-ios --features gmp-mpfr-sys/force-cross"
        echo "   2. Or use a different build environment"
    else
        echo "💻 Intel Mac detected - building for both iOS targets"
        # On Intel Mac, build for both targets
        cargo build --release --target x86_64-apple-ios
        cargo build --release --target aarch64-apple-ios
    fi
    
    echo "✅ iOS builds completed"
else
    echo "⚠️  Xcode not found, skipping iOS builds"
fi

# Show generated files
echo ""
echo "📁 Generated files:"
find target -name "libdkg_node_ffi.*" 2>/dev/null | head -10

echo ""
echo "🎉 Build completed successfully!"
echo ""
echo "📋 Usage:"
echo "   - macOS: target/release/libdkg_node_ffi.dylib"
if [[ "$PLATFORM" == "arm64" ]]; then
    echo "   - iOS: Not built (cross-compilation issues)"
    echo "   💡 For iOS on Apple Silicon, try:"
    echo "      cargo build --release --target aarch64-apple-ios --features gmp-mpfr-sys/force-cross"
else
    echo "   - iOS Simulator: target/x86_64-apple-ios/release/libdkg_node_ffi.a"
    echo "   - iOS Device: target/aarch64-apple-ios/release/libdkg_node_ffi.a"
fi
echo ""
echo "📖 See README.md for Swift integration instructions" 