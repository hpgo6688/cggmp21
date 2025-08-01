use std::ffi::{c_char, CStr, CString};
use std::ptr;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use base64::Engine;

mod dkg;
mod network;
mod types;

use dkg::*;
use network::*;
use types::*;

// FFI Error handling
#[repr(C)]
pub struct DkgResult {
    success: bool,
    error_message: *mut c_char,
    public_key: *mut c_char,
    node_index: u16,
    total_nodes: usize,
}

impl DkgResult {
    fn new() -> Self {
        DkgResult {
            success: false,
            error_message: ptr::null_mut(),
            public_key: ptr::null_mut(),
            node_index: 0,
            total_nodes: 0,
        }
    }

    fn set_error(&mut self, message: String) {
        self.success = false;
        self.error_message = CString::new(message).unwrap().into_raw();
    }

    fn set_success(&mut self, key_share: cggmp21::IncompleteKeyShare<cggmp21::supported_curves::Secp256k1>, node_id: u16, total: usize) {
        self.success = true;
        self.node_index = key_share.i;
        self.total_nodes = total;
        
        // Convert public key to string
        let public_key = key_share.shared_public_key();
        let public_key_str = format!("{:?}", public_key);
        self.public_key = CString::new(public_key_str).unwrap().into_raw();
    }
}

// FFI Functions

#[no_mangle]
pub extern "C" fn dkg_node_create() -> *mut Arc<Mutex<DkgState>> {
    let state = Arc::new(Mutex::new(DkgState::new(0, 2, 1)));
    Box::into_raw(Box::new(state))
}

#[no_mangle]
pub extern "C" fn dkg_node_destroy(state: *mut Arc<Mutex<DkgState>>) {
    if !state.is_null() {
        unsafe {
            let _ = Box::from_raw(state);
        }
    }
}

#[no_mangle]
pub extern "C" fn dkg_node_run_session(
    node_id: u16,
    relay_url: *const c_char,
    total_nodes: usize,
    session_id: *const c_char,
) -> *mut DkgResult {
    let mut result = DkgResult::new();
    
    // Convert C strings to Rust strings
    let relay_url = unsafe {
        if relay_url.is_null() {
            result.set_error("Relay URL is null".to_string());
            return Box::into_raw(Box::new(result));
        }
        match CStr::from_ptr(relay_url).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                result.set_error("Invalid relay URL encoding".to_string());
                return Box::into_raw(Box::new(result));
            }
        }
    };
    
    let session_id = unsafe {
        if session_id.is_null() {
            result.set_error("Session ID is null".to_string());
            return Box::into_raw(Box::new(result));
        }
        match CStr::from_ptr(session_id).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                result.set_error("Invalid session ID encoding".to_string());
                return Box::into_raw(Box::new(result));
            }
        }
    };

    // Create runtime and run DKG session
    let rt = Runtime::new().unwrap();
    let result = rt.block_on(async {
        let state = Arc::new(Mutex::new(DkgState::new(node_id, 2, total_nodes)));
        
        println!("Node {} starting DKG session: {}", node_id, session_id);

        // Round 1: Send broadcast messages
        {
            let mut s = state.lock().unwrap();
            let msg_bytes = s.execute_round1();
            drop(s);

            let msg = DkgMessage {
                session_id: session_id.clone(),
                from: node_id,
                to: None,
                round: 1,
                payload: base64::engine::general_purpose::STANDARD.encode(&msg_bytes),
            };
            send_to_relay(&relay_url, &msg).await;
            println!("Node {}: Round 1 message sent", node_id);
        }

        // Round 1: Fetch messages from other nodes
        loop {
            let msgs = fetch_from_relay(&relay_url, &session_id, 1, node_id).await;
            let mut s = state.lock().unwrap();
            for m in msgs {
                if m.from != node_id && !s.received_round1.contains_key(&m.from) {
                    let data = base64::engine::general_purpose::STANDARD.decode(&m.payload).unwrap();
                    s.handle_round1(m.from, data);
                }
            }

            if s.is_round1_complete(total_nodes) {
                println!("Node {}: Round 1 complete", node_id);
                break;
            }

            drop(s);
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        // Round 2: Send private messages to other nodes
        {
            let mut s = state.lock().unwrap();
            let messages = s.execute_round2(total_nodes);
            drop(s);

            for (to, msg_bytes) in messages {
                let msg = DkgMessage {
                    session_id: session_id.clone(),
                    from: node_id,
                    to: Some(to),
                    round: 2,
                    payload: base64::engine::general_purpose::STANDARD.encode(&msg_bytes),
                };
                send_to_relay(&relay_url, &msg).await;
            }
            println!("Node {}: Round 2 messages sent", node_id);
        }

        // Round 2: Fetch private messages
        loop {
            let msgs = fetch_from_relay(&relay_url, &session_id, 2, node_id).await;
            let mut s = state.lock().unwrap();
            for m in msgs {
                if m.from != node_id && !s.received_round2.contains_key(&m.from) {
                    let data = base64::engine::general_purpose::STANDARD.decode(&m.payload).unwrap();
                    s.handle_round2(m.from, data);
                }
            }

            if s.is_round2_complete(total_nodes) {
                println!("Node {}: Round 2 complete", node_id);
                break;
            }

            drop(s);
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        // Round 3: Broadcast messages
        {
            let mut s = state.lock().unwrap();
            let msg_bytes = s.execute_round3();
            drop(s);

            let msg = DkgMessage {
                session_id: session_id.clone(),
                from: node_id,
                to: None,
                round: 3,
                payload: base64::engine::general_purpose::STANDARD.encode(&msg_bytes),
            };
            send_to_relay(&relay_url, &msg).await;
            println!("Node {}: Round 3 message sent", node_id);
        }

        // Round 3: Fetch broadcast messages
        loop {
            let msgs = fetch_from_relay(&relay_url, &session_id, 3, node_id).await;
            let mut s = state.lock().unwrap();
            for m in msgs {
                if m.from != node_id && !s.received_round3.contains_key(&m.from) {
                    let data = base64::engine::general_purpose::STANDARD.decode(&m.payload).unwrap();
                    s.handle_round3(m.from, data);
                }
            }

            if s.is_round3_complete(total_nodes) {
                println!("Node {}: Round 3 complete", node_id);
                break;
            }

            drop(s);
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        // Generate final result
        println!("🔍 节点 {} 开始生成一致密钥共享...", node_id);
        let key_share = {
            let mut s = state.lock().unwrap();
            s.finalize(&session_id)
        };
        
        let mut result = DkgResult::new();
        if let Some(key_share) = key_share {
            println!("🎉 节点 {} DKG 协议完成，获得一致密钥共享！", node_id);
            result.set_success(key_share, node_id, total_nodes);
        } else {
            println!("❌ 节点 {} DKG 协议失败", node_id);
            result.set_error("DKG protocol failed".to_string());
        }
        
        result
    });

    Box::into_raw(Box::new(result))
}

#[no_mangle]
pub extern "C" fn dkg_result_destroy(result: *mut DkgResult) {
    if !result.is_null() {
        unsafe {
            let result = Box::from_raw(result);
            if !result.error_message.is_null() {
                let _ = CString::from_raw(result.error_message);
            }
            if !result.public_key.is_null() {
                let _ = CString::from_raw(result.public_key);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn dkg_result_get_success(result: *const DkgResult) -> bool {
    if result.is_null() {
        return false;
    }
    unsafe { (*result).success }
}

#[no_mangle]
pub extern "C" fn dkg_result_get_error_message(result: *const DkgResult) -> *const c_char {
    if result.is_null() {
        return ptr::null();
    }
    unsafe { (*result).error_message }
}

#[no_mangle]
pub extern "C" fn dkg_result_get_public_key(result: *const DkgResult) -> *const c_char {
    if result.is_null() {
        return ptr::null();
    }
    unsafe { (*result).public_key }
}

#[no_mangle]
pub extern "C" fn dkg_result_get_node_index(result: *const DkgResult) -> u16 {
    if result.is_null() {
        return 0;
    }
    unsafe { (*result).node_index }
}

#[no_mangle]
pub extern "C" fn dkg_result_get_total_nodes(result: *const DkgResult) -> usize {
    if result.is_null() {
        return 0;
    }
    unsafe { (*result).total_nodes }
} 