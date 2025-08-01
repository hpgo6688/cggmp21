# TdkgCore.xcframework 使用说明

## ✅ 成功生成的 xcframework

构建脚本已成功生成了 `TdkgCore.xcframework`，包含以下内容：

### 📁 文件结构
```
TdkgCore.xcframework/
├── Info.plist                           # 框架信息配置
├── ios-arm64/                           # iOS 真机版本
│   ├── Headers/
│   │   └── tdkg_core.h                  # C 头文件
│   └── libtdkg_core_device.a            # 真机静态库 (15.9MB)
└── ios-arm64_x86_64-simulator/         # iOS 模拟器版本
    ├── Headers/
    │   └── tdkg_core.h                  # C 头文件
    └── libtdkg_core_sim.a               # 模拟器静态库 (31.8MB)
```

### 🔧 支持的平台
- **iOS 真机**: arm64 架构
- **iOS 模拟器**: arm64 + x86_64 架构 (通用二进制)

## 🚀 在 Xcode 项目中使用

### 1. 添加框架到项目

1. 将 `TdkgCore.xcframework` 拖拽到 Xcode 项目中
2. 在 "Add to target" 中选择你的应用目标
3. 确保 "Copy items if needed" 已勾选

### 2. 链接设置

在 Xcode 项目设置中：
1. 选择你的应用目标
2. 进入 "Build Settings"
3. 搜索 "Other Linker Flags"
4. 添加 `-ltdkg_core`

### 3. Swift 代码示例

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
class TdkgCore {
    static func add(_ a: Int, _ b: Int) -> Int {
        return Int(add(Int32(a), Int32(b)))
    }
    
    static func multiply(_ a: Int, _ b: Int) -> Int {
        return Int(multiply(Int32(a), Int32(b)))
    }
    
    static func getVersion() -> String? {
        guard let versionPtr = getVersion() else { return nil }
        let version = String(cString: versionPtr)
        freeString(versionPtr)
        return version
    }
    
    static func calculateSum(_ numbers: [Int]) -> Int {
        let int32Numbers = numbers.map { Int32($0) }
        return Int(int32Numbers.withUnsafeBufferPointer { ptr in
            calculateSum(ptr.baseAddress, ptr.count)
        })
    }
    
    static func createHelloMessage(name: String?) -> String? {
        let namePtr: UnsafePointer<Int8>?
        
        if let name = name {
            namePtr = name.cString(using: .utf8)
        } else {
            namePtr = nil
        }
        
        guard let messagePtr = createHelloMessage(namePtr) else { return nil }
        let message = String(cString: messagePtr)
        freeString(messagePtr)
        return message
    }
}

// 测试代码
func testTdkgCore() {
    print("🚀 Testing TdkgCore.xcframework")
    
    // 基本运算
    let sum = TdkgCore.add(5, 7)
    print("5 + 7 = \(sum)")
    
    let product = TdkgCore.multiply(4, 6)
    print("4 * 6 = \(product)")
    
    // 版本信息
    if let version = TdkgCore.getVersion() {
        print("Version: \(version)")
    }
    
    // 数组计算
    let numbers = [1, 2, 3, 4, 5]
    let arraySum = TdkgCore.calculateSum(numbers)
    print("Sum of \(numbers) = \(arraySum)")
    
    // 字符串处理
    if let message = TdkgCore.createHelloMessage(name: "Alice") {
        print("Message: \(message)")
    }
}
```

## 📋 验证清单

- [x] ✅ xcframework 生成成功
- [x] ✅ 包含真机和模拟器版本
- [x] ✅ 头文件正确包含
- [x] ✅ 静态库文件完整
- [x] ✅ Info.plist 配置正确
- [x] ✅ 支持多架构 (arm64, x86_64)

## 🔍 故障排除

### 常见问题

1. **链接错误**: 确保在 "Other Linker Flags" 中添加了 `-ltdkg_core`

2. **找不到符号**: 确保 xcframework 已正确添加到项目中

3. **架构不匹配**: 确保 xcframework 支持目标架构

4. **内存泄漏**: 记得调用 `free_string` 释放 Rust 分配的内存

### 调试技巧

```bash
# 检查 xcframework 支持的架构
lipo -info TdkgCore.xcframework/ios-arm64/libtdkg_core_device.a
lipo -info TdkgCore.xcframework/ios-arm64_x86_64-simulator/libtdkg_core_sim.a

# 检查符号表
nm TdkgCore.xcframework/ios-arm64/libtdkg_core_device.a | grep add
```

## 🎯 下一步

现在你可以：
1. 将 `TdkgCore.xcframework` 集成到你的 iOS 项目中
2. 使用提供的 Swift 包装类调用 Rust 函数
3. 根据需要扩展 Rust 库的功能
4. 添加更多的 FFI 函数和数据结构

这个 xcframework 为真正的 DKG 实现提供了完整的基础！ 