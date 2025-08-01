# DKG Node FFI Integration Guide

This guide explains how to integrate the DKG Node FFI library into your Swift projects.

## Overview

The `dkg-node-ffi` library provides a C-compatible interface for the DKG (Distributed Key Generation) protocol, making it easy to use from Swift applications. The library handles the complex DKG protocol internally while providing a simple FFI interface.

## Quick Start

### 1. Build the Library

```bash
cd dkg-node-ffi
./build.sh
```

This will generate:
- `target/release/libdkg_node_ffi.dylib` (macOS)
- `target/aarch64-apple-ios/release/libdkg_node_ffi.a` (iOS Device)
- `target/x86_64-apple-ios/release/libdkg_node_ffi.a` (iOS Simulator, Intel Mac only)

**Note**: On Apple Silicon Macs, only the iOS Device target is built due to cross-compilation limitations.

### 2. Add to Xcode Project

1. **Add the library files**:
   - Drag `libdkg_node_ffi.dylib` to your Xcode project for macOS
   - Drag `libdkg_node_ffi.a` files to your Xcode project for iOS

2. **Add the header file**:
   - Add `dkg_node_ffi.h` to your project

3. **Configure linking**:
   - In Build Settings, add the library path to "Library Search Paths"
   - In Build Phases, add the library to "Link Binary With Libraries"

### 3. Create Bridging Header

Create a bridging header file (e.g., `YourProject-Bridging-Header.h`) and add:

```objc
#import "dkg_node_ffi.h"
```

### 4. Use in Swift

```swift
import Foundation

class DkgManager {
    private let dkgNode = DkgNodeSwift()
    
    func runDkgSession(
        nodeId: UInt16,
        relayUrl: String,
        totalNodes: Int,
        sessionId: String
    ) -> DkgSessionResult {
        return dkgNode.runSession(
            nodeId: nodeId,
            relayUrl: relayUrl,
            totalNodes: totalNodes,
            sessionId: sessionId
        )
    }
}

// Usage
let manager = DkgManager()
let result = manager.runDkgSession(
    nodeId: 0,
    relayUrl: "http://localhost:8080",
    totalNodes: 3,
    sessionId: "my-session-123"
)

if result.isSuccess {
    print("DKG completed! Public key: \(result.publicKey ?? "")")
} else {
    print("DKG failed: \(result.errorMessage ?? "")")
}
```

## API Reference

### Core Functions

#### `dkg_node_create()`
Creates a new DKG node instance.

**Returns**: Pointer to DKG state (must be freed with `dkg_node_destroy`)

#### `dkg_node_destroy(state)`
Destroys a DKG node instance and frees memory.

**Parameters**:
- `state`: Pointer to DKG state

#### `dkg_node_run_session(node_id, relay_url, total_nodes, session_id)`
Runs a complete DKG session.

**Parameters**:
- `node_id`: Node identifier (0-based)
- `relay_url`: Relay server URL
- `total_nodes`: Total number of nodes in the session
- `session_id`: Unique session identifier

**Returns**: Pointer to DKG result (must be freed with `dkg_result_destroy`)

### Result Accessor Functions

#### `dkg_result_get_success(result)`
Returns whether the DKG session was successful.

#### `dkg_result_get_error_message(result)`
Returns error message if the session failed.

#### `dkg_result_get_public_key(result)`
Returns the generated public key as a string.

#### `dkg_result_get_node_index(result)`
Returns the node index in the DKG session.

#### `dkg_result_get_total_nodes(result)`
Returns the total number of nodes in the session.

#### `dkg_result_destroy(result)`
Frees the result object and its associated memory.

## Swift Wrapper

The `DkgNodeSwift.swift` file provides a Swift wrapper that handles memory management automatically:

```swift
public class DkgNodeSwift {
    public func runSession(
        nodeId: UInt16,
        relayUrl: String,
        totalNodes: Int,
        sessionId: String
    ) -> DkgSessionResult
}

public class DkgSessionResult {
    public var isSuccess: Bool
    public var errorMessage: String?
    public var publicKey: String?
    public var nodeIndex: UInt16
    public var totalNodes: Int
}
```

## Memory Management

The Swift wrapper automatically handles memory management through ARC. However, if you use the C functions directly, remember to:

1. Call `dkg_node_destroy()` for node instances
2. Call `dkg_result_destroy()` for result objects

## Error Handling

The library provides comprehensive error handling:

1. **Null pointer checks**: All functions check for null pointers
2. **String encoding validation**: C strings are validated for UTF-8 encoding
3. **Error reporting**: Failed operations return detailed error messages

## Multi-Node Setup

For a complete DKG setup, you need multiple nodes running simultaneously:

```swift
// Node 0
let node0 = DkgNodeSwift()
let result0 = node0.runSession(
    nodeId: 0,
    relayUrl: "http://localhost:8080",
    totalNodes: 3,
    sessionId: "multi-node-session"
)

// Node 1
let node1 = DkgNodeSwift()
let result1 = node1.runSession(
    nodeId: 1,
    relayUrl: "http://localhost:8080",
    totalNodes: 3,
    sessionId: "multi-node-session"
)

// Node 2
let node2 = DkgNodeSwift()
let result2 = node2.runSession(
    nodeId: 2,
    relayUrl: "http://localhost:8080",
    totalNodes: 3,
    sessionId: "multi-node-session"
)
```

## Relay Server

The DKG protocol requires a relay server for communication between nodes. You can use the provided relay server:

```bash
cd ../relay-server
cargo run
```

The relay server runs on `http://localhost:8080` by default.

## Troubleshooting

### Common Issues

1. **Library not found**: Ensure the library is properly linked in Xcode
2. **Header not found**: Check that the bridging header is configured correctly
3. **Memory leaks**: Use the Swift wrapper to avoid manual memory management
4. **Network errors**: Ensure the relay server is running and accessible

### Debug Tips

1. **Enable logging**: The library provides detailed console output
2. **Check network**: Verify relay server connectivity
3. **Validate parameters**: Ensure all parameters are within valid ranges

## Performance Considerations

1. **Async operations**: DKG sessions are asynchronous and may take time
2. **Memory usage**: Large numbers of nodes increase memory usage
3. **Network latency**: Relay server location affects performance

## Security Notes

1. **Key generation**: The library uses cryptographically secure random number generation
2. **Network security**: Consider using HTTPS for relay server communication
3. **Key storage**: Implement secure storage for generated keys

## Examples

See the `test_ffi.rs` file for a complete example of using the FFI interface directly.

## Support

For issues and questions:
1. Check the README.md file
2. Review the test examples
3. Check the console output for error messages 