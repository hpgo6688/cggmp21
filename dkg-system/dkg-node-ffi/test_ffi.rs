use std::ffi::{CStr, CString};
use std::ptr;

// Import the FFI functions
extern "C" {
    fn dkg_node_create() -> *mut std::ffi::c_void;
    fn dkg_node_destroy(state: *mut std::ffi::c_void);
    fn dkg_node_run_session(
        node_id: u16,
        relay_url: *const i8,
        total_nodes: usize,
        session_id: *const i8,
    ) -> *mut std::ffi::c_void;
    fn dkg_result_destroy(result: *mut std::ffi::c_void);
    fn dkg_result_get_success(result: *const std::ffi::c_void) -> bool;
    fn dkg_result_get_error_message(result: *const std::ffi::c_void) -> *const i8;
    fn dkg_result_get_public_key(result: *const std::ffi::c_void) -> *const i8;
    fn dkg_result_get_node_index(result: *const std::ffi::c_void) -> u16;
    fn dkg_result_get_total_nodes(result: *const std::ffi::c_void) -> usize;
}

fn main() {
    println!("Testing DKG Node FFI interface...");
    
    // Test 1: Create and destroy node
    println!("Test 1: Creating DKG node...");
    let node = unsafe { dkg_node_create() };
    if !node.is_null() {
        println!("✅ Node created successfully");
        unsafe { dkg_node_destroy(node) };
        println!("✅ Node destroyed successfully");
    } else {
        println!("❌ Failed to create node");
        return;
    }
    
    // Test 2: Run DKG session (this will fail without relay server, but tests the interface)
    println!("\nTest 2: Testing DKG session interface...");
    let relay_url = CString::new("http://localhost:8080").unwrap();
    let session_id = CString::new("test-session-123").unwrap();
    
    let result = unsafe {
        dkg_node_run_session(
            0, // node_id
            relay_url.as_ptr(),
            3, // total_nodes
            session_id.as_ptr(),
        )
    };
    
    if !result.is_null() {
        let success = unsafe { dkg_result_get_success(result) };
        let node_index = unsafe { dkg_result_get_node_index(result) };
        let total_nodes = unsafe { dkg_result_get_total_nodes(result) };
        
        println!("✅ DKG session completed");
        println!("   Success: {}", success);
        println!("   Node index: {}", node_index);
        println!("   Total nodes: {}", total_nodes);
        
        if success {
            let public_key_ptr = unsafe { dkg_result_get_public_key(result) };
            if !public_key_ptr.is_null() {
                let public_key = unsafe { CStr::from_ptr(public_key_ptr) };
                println!("   Public key: {}", public_key.to_string_lossy());
            }
        } else {
            let error_ptr = unsafe { dkg_result_get_error_message(result) };
            if !error_ptr.is_null() {
                let error_msg = unsafe { CStr::from_ptr(error_ptr) };
                println!("   Error: {}", error_msg.to_string_lossy());
            }
        }
        
        unsafe { dkg_result_destroy(result) };
        println!("✅ Result destroyed successfully");
    } else {
        println!("❌ Failed to run DKG session");
    }
    
    println!("\n🎉 FFI interface test completed!");
} 