use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DkgMessage {
    pub session_id: String,
    pub from: u16,
    pub to: Option<u16>,
    pub round: u8,
    pub payload: String,
} 