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

// Swift 包装类，提供更友好的 API
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

// 演示函数
func runDemo() {
    print("🚀 tdkg_core Demo")
    print("================\n")
    
    // 基本数学运算
    print("📊 Basic Math Operations:")
    let sum = TdkgCore.add(5, 7)
    print("  5 + 7 = \(sum)")
    
    let product = TdkgCore.multiply(4, 6)
    print("  4 * 6 = \(product)")
    
    // 获取版本信息
    print("\n📋 Version Info:")
    if let version = TdkgCore.getVersion() {
        print("  \(version)")
    }
    
    // 数组计算
    print("\n📈 Array Operations:")
    let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    let arraySum = TdkgCore.calculateSum(numbers)
    print("  Sum of \(numbers) = \(arraySum)")
    
    // 字符串处理
    print("\n💬 String Operations:")
    if let message1 = TdkgCore.createHelloMessage(name: "Alice") {
        print("  \(message1)")
    }
    
    if let message2 = TdkgCore.createHelloMessage(name: nil) {
        print("  \(message2)")
    }
    
    if let message3 = TdkgCore.createHelloMessage(name: "World") {
        print("  \(message3)")
    }
    
    print("\n✅ Demo completed successfully!")
}

// 如果直接运行此文件
if CommandLine.arguments.contains("--demo") {
    runDemo()
} 