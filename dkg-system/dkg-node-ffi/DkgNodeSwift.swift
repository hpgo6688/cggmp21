import Foundation

// Swift wrapper for DKG Node FFI
public class DkgNodeSwift {
    
    // MARK: - Properties
    private var dkgState: UnsafeMutableRawPointer?
    
    // MARK: - Initialization
    public init() {
        dkgState = dkg_node_create()
    }
    
    deinit {
        if let state = dkgState {
            dkg_node_destroy(state)
        }
    }
    
    // MARK: - DKG Session Management
    public func runSession(
        nodeId: UInt16,
        relayUrl: String,
        totalNodes: Int,
        sessionId: String
    ) -> DkgSessionResult {
        
        let result = dkg_node_run_session(
            nodeId,
            relayUrl,
            UInt(totalNodes),
            sessionId
        )
        
        return DkgSessionResult(result: result)
    }
}

// MARK: - DKG Session Result
public class DkgSessionResult {
    private let result: UnsafeMutableRawPointer?
    
    init(result: UnsafeMutableRawPointer?) {
        self.result = result
    }
    
    deinit {
        if let result = result {
            dkg_result_destroy(result)
        }
    }
    
    public var isSuccess: Bool {
        guard let result = result else { return false }
        return dkg_result_get_success(result)
    }
    
    public var errorMessage: String? {
        guard let result = result else { return nil }
        guard let errorPtr = dkg_result_get_error_message(result) else { return nil }
        return String(cString: errorPtr)
    }
    
    public var publicKey: String? {
        guard let result = result else { return nil }
        guard let keyPtr = dkg_result_get_public_key(result) else { return nil }
        return String(cString: keyPtr)
    }
    
    public var nodeIndex: UInt16 {
        guard let result = result else { return 0 }
        return dkg_result_get_node_index(result)
    }
    
    public var totalNodes: Int {
        guard let result = result else { return 0 }
        return Int(dkg_result_get_total_nodes(result))
    }
}

// MARK: - C Function Declarations
// These would typically be in a bridging header file
@_silgen_name("dkg_node_create")
func dkg_node_create() -> UnsafeMutableRawPointer?

@_silgen_name("dkg_node_destroy")
func dkg_node_destroy(_ state: UnsafeMutableRawPointer?)

@_silgen_name("dkg_node_run_session")
func dkg_node_run_session(
    _ nodeId: UInt16,
    _ relayUrl: UnsafePointer<Int8>?,
    _ totalNodes: UInt,
    _ sessionId: UnsafePointer<Int8>?
) -> UnsafeMutableRawPointer?

@_silgen_name("dkg_result_destroy")
func dkg_result_destroy(_ result: UnsafeMutableRawPointer?)

@_silgen_name("dkg_result_get_success")
func dkg_result_get_success(_ result: UnsafeRawPointer?) -> Bool

@_silgen_name("dkg_result_get_error_message")
func dkg_result_get_error_message(_ result: UnsafeRawPointer?) -> UnsafePointer<Int8>?

@_silgen_name("dkg_result_get_public_key")
func dkg_result_get_public_key(_ result: UnsafeRawPointer?) -> UnsafePointer<Int8>?

@_silgen_name("dkg_result_get_node_index")
func dkg_result_get_node_index(_ result: UnsafeRawPointer?) -> UInt16

@_silgen_name("dkg_result_get_total_nodes")
func dkg_result_get_total_nodes(_ result: UnsafeRawPointer?) -> UInt 