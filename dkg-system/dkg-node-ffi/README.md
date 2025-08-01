# DKG Node FFI

This is the FFI (Foreign Function Interface) version of the DKG node, designed to be used from Swift applications.

## Features

- **FFI Interface**: C-compatible interface for cross-language integration
- **Swift Support**: Complete Swift wrapper class for easy integration
- **Memory Safety**: Proper memory management with automatic cleanup
- **Error Handling**: Comprehensive error reporting through FFI

## Building

### Build the Rust Library

```bash
cd dkg-node-ffi
cargo build --release
```

This will generate:
- `libdkg_node_ffi.dylib` (macOS)
- `libdkg_node_ffi.a` (static library)

### Build for iOS

```bash
# On Apple Silicon Mac (M1/M2)
cargo build --release --target aarch64-apple-ios

# On Intel Mac
cargo build --release --target aarch64-apple-ios
cargo build --release --target x86_64-apple-ios
```

**Note**: Cross-compilation from Apple Silicon to x86_64 is not supported due to `gmp-mpfr-sys` limitations. On Apple Silicon Macs, only the iOS Device (arm64) target is built.

## Swift Integration

### 1. Add the Library to Your Xcode Project

1. Add the built library to your Xcode project
2. Add the header file `dkg_node_ffi.h` to your project
3. Link against the library in your project settings

### 2. Create a Bridging Header

Create a bridging header file (e.g., `YourProject-Bridging-Header.h`) and add:

```objc
#import "dkg_node_ffi.h"
```

### 3. Use the Swift Wrapper

```swift
import Foundation

// Create DKG node instance
let dkgNode = DkgNodeSwift()

// Run DKG session
let result = dkgNode.runSession(
    nodeId: 1,
    relayUrl: "http://localhost:8080",
    totalNodes: 3,
    sessionId: "session-123"
)

// Check result
if result.isSuccess {
    print("DKG session completed successfully!")
    print("Public key: \(result.publicKey ?? "N/A")")
    print("Node index: \(result.nodeIndex)")
    print("Total nodes: \(result.totalNodes)")
} else {
    print("DKG session failed: \(result.errorMessage ?? "Unknown error")")
}
```

## C API Reference

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

## Memory Management

The FFI interface uses manual memory management. Always call the corresponding destroy functions:

- `dkg_node_destroy()` for node instances
- `dkg_result_destroy()` for result objects

The Swift wrapper automatically handles memory management through ARC.

## Error Handling

The FFI interface provides error information through the result object:

1. Check `isSuccess` or `dkg_result_get_success()` first
2. If false, get the error message with `errorMessage` or `dkg_result_get_error_message()`

## Example Usage

### Basic Usage

```swift
let dkgNode = DkgNodeSwift()

let result = dkgNode.runSession(
    nodeId: 0,
    relayUrl: "http://localhost:8080",
    totalNodes: 3,
    sessionId: "test-session"
)

if result.isSuccess {
    print("Success! Public key: \(result.publicKey ?? "")")
} else {
    print("Failed: \(result.errorMessage ?? "")")
}
```

### Multi-node Setup

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

## Requirements

- Rust 1.70+
- Xcode 14+ (for iOS development)
- macOS 12+ (for macOS development)

## License

Same as the original dkg-node project. 