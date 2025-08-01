# tdkg_core Demo

这是一个演示如何将 Rust 库编译为 iOS 可用的 `.xcframework` 的示例项目。

## 项目结构

```
tdkg_core_demo/
├── Cargo.toml          # Rust 项目配置
├── cbindgen.toml       # 头文件生成配置
├── build.sh            # 构建脚本
├── src/
│   └── lib.rs          # Rust 库源码
├── include/            # 生成的头文件目录
└── README.md           # 项目说明
```

## 功能

这个 demo 包含以下 FFI 函数：

- `add(a, b)` - 加法运算
- `multiply(a, b)` - 乘法运算
- `get_version()` - 获取版本信息
- `free_string(ptr)` - 释放字符串内存
- `calculate_sum(array, length)` - 计算数组总和
- `create_hello_message(name)` - 创建问候消息

## 构建步骤

### 1. 安装依赖

确保已安装 Rust 和 Xcode：

```bash
# 安装 Rust (如果未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 cbindgen
cargo install cbindgen
```

### 2. 构建项目

```bash
# 运行构建脚本
./build.sh
```

构建脚本会自动：
- 添加 iOS 目标平台
- 生成 C 头文件
- 编译多平台静态库
- 创建 `.xcframework`

### 3. 输出文件

构建完成后会生成：

- `TdkgCore.xcframework/` - iOS 框架
- `include/tdkg_core.h` - C 头文件
- `target/ios/libtdkg_core_device.a` - 真机静态库
- `target/ios/libtdkg_core_sim.a` - 模拟器静态库

## Swift 使用示例

```swift
import Foundation

// 导入 Rust 函数
@_silgen_name("add")
func add(_ a: Int32, _ b: Int32) -> Int32

@_silgen_name("multiply")
func multiply(_ a: Int32, _ b: Int32) -> Int32

@_silgen_name("get_version")
func getVersion() -> UnsafeMutablePointer<Int8>?

@_silgen_name("free_string")
func freeString(_ ptr: UnsafeMutablePointer<Int8>?)

@_silgen_name("calculate_sum")
func calculateSum(_ array: UnsafePointer<Int32>?, _ length: Int) -> Int32

@_silgen_name("create_hello_message")
func createHelloMessage(_ name: UnsafePointer<Int8>?) -> UnsafeMutablePointer<Int8>?

// 使用示例
let result1 = add(5, 7)
print("5 + 7 = \(result1)")

let result2 = multiply(4, 6)
print("4 * 6 = \(result2)")

if let versionPtr = getVersion() {
    let version = String(cString: versionPtr)
    print("Version: \(version)")
    freeString(versionPtr)
}

let numbers: [Int32] = [1, 2, 3, 4, 5]
let sum = numbers.withUnsafeBufferPointer { ptr in
    calculateSum(ptr.baseAddress, ptr.count)
}
print("Sum of array: \(sum)")

if let namePtr = "Alice".cString(using: .utf8) {
    let messagePtr = createHelloMessage(namePtr)
    if let messagePtr = messagePtr {
        let message = String(cString: messagePtr)
        print("Message: \(message)")
        freeString(messagePtr)
    }
}
```

## 手动构建命令

如果不想使用构建脚本，可以手动执行：

```bash
# 添加目标平台
rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim

# 生成头文件
cbindgen --config cbindgen.toml --crate tdkg_core --output include/tdkg_core.h

# 编译各平台
cargo build --target aarch64-apple-ios --release
cargo build --target x86_64-apple-ios --release
cargo build --target aarch64-apple-ios-sim --release

# 合并模拟器库
lipo -create \
  target/x86_64-apple-ios/release/libtdkg_core.a \
  target/aarch64-apple-ios-sim/release/libtdkg_core.a \
  -output libtdkg_core_sim.a

# 创建 xcframework
xcodebuild -create-xcframework \
  -library target/aarch64-apple-ios/release/libtdkg_core.a -headers include \
  -library libtdkg_core_sim.a -headers include \
  -output TdkgCore.xcframework
```

## 注意事项

1. 确保 Xcode 命令行工具已安装
2. 在 macOS 上运行构建脚本
3. 生成的 `.xcframework` 可以直接拖拽到 Xcode 项目中使用
4. 记得在 Swift 代码中调用 `free_string` 来释放 Rust 分配的内存 