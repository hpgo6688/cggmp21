#!/bin/bash

set -e

echo "🧪 Testing tdkg_core build process..."

# 检查项目文件
echo "📁 Checking project structure..."
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Cargo.toml not found"
    exit 1
fi

if [ ! -f "src/lib.rs" ]; then
    echo "❌ src/lib.rs not found"
    exit 1
fi

if [ ! -f "cbindgen.toml" ]; then
    echo "❌ cbindgen.toml not found"
    exit 1
fi

echo "✅ Project structure looks good"

# 测试本地编译
echo "🔨 Testing local compilation..."
cargo check
echo "✅ Local compilation successful"

# 测试头文件生成
echo "📄 Testing header generation..."
if command -v cbindgen &> /dev/null; then
    cbindgen --config cbindgen.toml --crate tdkg_core --output include/tdkg_core.h
    if [ -f "include/tdkg_core.h" ]; then
        echo "✅ Header file generated successfully"
        echo "📄 Header file preview:"
        head -10 include/tdkg_core.h
    else
        echo "❌ Header file generation failed"
        exit 1
    fi
else
    echo "⚠️  cbindgen not installed, skipping header generation test"
fi

# 测试一个目标平台的编译
echo "📱 Testing iOS target compilation..."
rustup target add aarch64-apple-ios
cargo build --target aarch64-apple-ios --release
if [ -f "target/aarch64-apple-ios/release/libtdkg_core.a" ]; then
    echo "✅ iOS target compilation successful"
else
    echo "❌ iOS target compilation failed"
    exit 1
fi

echo "🎉 All tests passed! The project is ready for full build."
echo ""
echo "To build the complete xcframework, run:"
echo "  ./build.sh" 