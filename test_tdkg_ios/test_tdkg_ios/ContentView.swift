//
//  ContentView.swift
//  test_tdkg_ios
//
//  Created by haotian.chen on 2025/8/1.
//

import SwiftUI

// 直接使用 xcframework 中的函数
struct ContentView: View {
    @State private var addResult: Int = 0
    @State private var multiplyResult: Int = 0
    @State private var versionInfo: String = "未获取"
    @State private var arraySum: Int = 0
    @State private var helloMessage: String = "未生成"
    @State private var inputName: String = ""
    @State private var inputArray: String = "1,2,3,4,5"
    @State private var inputA: String = "5"
    @State private var inputB: String = "7"
    
    var body: some View {
        ScrollView {
            VStack(spacing: 20) {
                // 标题
                Text("🚀 TdkgCore.xcframework 测试")
                    .font(.largeTitle)
                    .fontWeight(.bold)
                    .padding()
                
                // 版本信息
                GroupBox("📋 版本信息") {
                    VStack(alignment: .leading, spacing: 10) {
                        Text("当前版本: \(versionInfo)")
                            .font(.system(.body, design: .monospaced))
                        
                        Button("获取版本") {
                            // 直接调用 xcframework 中的 get_version 函数
                            if let versionPtr = get_version() {
                                versionInfo = String(cString: versionPtr)
                                free_string(versionPtr)
                            }
                        }
                        .buttonStyle(.borderedProminent)
                    }
                    .padding()
                }
                
                // 基本数学运算
                GroupBox("📊 基本数学运算") {
                    VStack(alignment: .leading, spacing: 10) {
                        HStack {
                            TextField("数字 A", text: $inputA)
                                .textFieldStyle(.roundedBorder)
                                .keyboardType(.numberPad)
                            
                            TextField("数字 B", text: $inputB)
                                .textFieldStyle(.roundedBorder)
                                .keyboardType(.numberPad)
                        }
                        
                        HStack {
                            Button("加法运算") {
                                if let a = Int(inputA), let b = Int(inputB) {
                                    // 直接调用 xcframework 中的 add 函数
                                    addResult = Int(add(Int32(a), Int32(b)))
                                }
                            }
                            .buttonStyle(.borderedProminent)
                            
                            Button("乘法运算") {
                                if let a = Int(inputA), let b = Int(inputB) {
                                    // 直接调用 xcframework 中的 multiply 函数
                                    multiplyResult = Int(multiply(Int32(a), Int32(b)))
                                }
                            }
                            .buttonStyle(.borderedProminent)
                        }
                        
                        VStack(alignment: .leading, spacing: 5) {
                            Text("加法结果: \(addResult)")
                                .font(.system(.body, design: .monospaced))
                            Text("乘法结果: \(multiplyResult)")
                                .font(.system(.body, design: .monospaced))
                        }
                    }
                    .padding()
                }
                
                // 数组计算
                GroupBox("📈 数组计算") {
                    VStack(alignment: .leading, spacing: 10) {
                        TextField("数组 (用逗号分隔)", text: $inputArray)
                            .textFieldStyle(.roundedBorder)
                        
                        Button("计算数组总和") {
                            let numbers = inputArray
                                .split(separator: ",")
                                .compactMap { Int($0.trimmingCharacters(in: .whitespaces)) }
                            
                            // 直接调用 xcframework 中的 calculate_sum 函数
                            let int32Numbers = numbers.map { Int32($0) }
                            arraySum = Int(int32Numbers.withUnsafeBufferPointer { ptr in
                                calculate_sum(ptr.baseAddress, UInt(ptr.count))
                            })
                        }
                        .buttonStyle(.borderedProminent)
                        
                        Text("数组总和: \(arraySum)")
                            .font(.system(.body, design: .monospaced))
                    }
                    .padding()
                }
                
                // 字符串处理
                GroupBox("💬 字符串处理") {
                    VStack(alignment: .leading, spacing: 10) {
                        TextField("输入姓名", text: $inputName)
                            .textFieldStyle(.roundedBorder)
                        
                        HStack {
                            Button("生成问候语") {
                                let name = inputName.isEmpty ? nil : inputName
                                
                                if let name = name {
                                    name.withCString { cString in
                                        if let messagePtr = create_hello_message(cString) {
                                            helloMessage = String(cString: messagePtr)
                                            free_string(messagePtr)
                                        }
                                    }
                                } else {
                                    if let messagePtr = create_hello_message(nil) {
                                        helloMessage = String(cString: messagePtr)
                                        free_string(messagePtr)
                                    }
                                }
                            }
                            .buttonStyle(.borderedProminent)
                            
                            Button("清空姓名") {
                                inputName = ""
                            }
                            .buttonStyle(.bordered)
                        }
                        
                        Text("问候语: \(helloMessage)")
                            .font(.system(.body, design: .monospaced))
                            .foregroundColor(.secondary)
                    }
                    .padding()
                }
                
                // 一键测试
                GroupBox("🧪 一键测试") {
                    VStack(alignment: .leading, spacing: 10) {
                        Button("运行所有测试") {
                            runAllTests()
                        }
                        .buttonStyle(.borderedProminent)
                        .frame(maxWidth: .infinity)
                        
                        Text("点击按钮运行所有测试功能")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                    .padding()
                }
                
                // 状态信息
                GroupBox("📊 状态信息") {
                    VStack(alignment: .leading, spacing: 5) {
                        Text("✅ TdkgCore.xcframework 已集成")
                        Text("✅ 直接使用头文件中的函数")
                        Text("✅ 无需手动声明接口")
                        Text("✅ 内存管理正确")
                        Text("✅ 多平台支持 (真机 + 模拟器)")
                    }
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .padding()
                }
            }
            .padding()
        }
    }
    
    private func runAllTests() {
        // 测试版本信息
        if let versionPtr = get_version() {
            versionInfo = String(cString: versionPtr)
            free_string(versionPtr)
        }
        
        // 测试数学运算
        addResult = Int(add(10, 20))
        multiplyResult = Int(multiply(6, 8))
        
        // 测试数组计算
        let testArray = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        let int32Numbers = testArray.map { Int32($0) }
        arraySum = Int(int32Numbers.withUnsafeBufferPointer { ptr in
            calculate_sum(ptr.baseAddress, UInt(ptr.count))
        })
        inputArray = testArray.map(String.init).joined(separator: ",")
        
        // 测试字符串处理
        if let messagePtr = create_hello_message("TdkgCore".cString(using: .utf8)) {
            helloMessage = String(cString: messagePtr)
            free_string(messagePtr)
        }
        
        // 更新输入值
        inputA = "10"
        inputB = "20"
        inputName = "TdkgCore"
    }
}

#Preview {
    ContentView()
}
