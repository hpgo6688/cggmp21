//! MPC Server Client implementation for CGGMP21 DKG and signing
//! 
//! This library provides a complete implementation of MPC (Multi-Party Computation)
//! server and client for distributed key generation (DKG) and threshold signing
//! using the CGGMP21 protocol with secp256k1 curve.

pub mod server;
pub mod client;
pub mod protocol;
pub mod storage;
pub mod types;
pub mod utils;

pub use server::MpcServer;
pub use client::MpcClient;
pub use types::*;
pub use types::{SessionId, PartyId, ProtocolType, ProtocolState}; 