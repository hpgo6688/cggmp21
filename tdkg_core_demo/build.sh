#!/bin/bash

set -e

echo "🚀 Building tdkg_core for iOS..."

# 检查是否安装了 Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust first."
    exit 1
fi

# 检查是否安装了 cbindgen
if ! command -v cbindgen &> /dev/null; then
    echo "📦 Installing cbindgen..."
    cargo install cbindgen
fi

# 添加 iOS 目标平台
echo "📱 Adding iOS targets..."
rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim

# 创建输出目录
mkdir -p target/ios

# 生成头文件
echo "📄 Generating header file..."
cbindgen --config cbindgen.toml --crate tdkg_core --output include/tdkg_core.h

# 编译 iOS 真机 (arm64)
echo "🔨 Building for iOS device (arm64)..."
cargo build --target aarch64-apple-ios --release

# 编译 iOS 模拟器 (x86_64)
echo "🔨 Building for iOS simulator (x86_64)..."
cargo build --target x86_64-apple-ios --release

# 编译 iOS 模拟器 (arm64)
echo "🔨 Building for iOS simulator (arm64)..."
cargo build --target aarch64-apple-ios-sim --release

# 合并模拟器静态库
echo "🔗 Merging simulator libraries..."
lipo -create \
  target/x86_64-apple-ios/release/libtdkg_core.a \
  target/aarch64-apple-ios-sim/release/libtdkg_core.a \
  -output target/ios/libtdkg_core_sim.a

# 复制真机库
cp target/aarch64-apple-ios/release/libtdkg_core.a target/ios/libtdkg_core_device.a

# 创建 xcframework
echo "📦 Creating xcframework..."
xcodebuild -create-xcframework \
  -library target/ios/libtdkg_core_device.a -headers include \
  -library target/ios/libtdkg_core_sim.a -headers include \
  -output TdkgCore.xcframework

echo "✅ Build completed successfully!"
echo "📁 Output files:"
echo "   - TdkgCore.xcframework/"
echo "   - include/tdkg_core.h"
echo "   - target/ios/libtdkg_core_device.a"
echo "   - target/ios/libtdkg_core_sim.a" 