use axum::{routing::{post, get}, Json, Router, extract::Path, serve};
use serde::{Serialize, Deserialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DkgMessage {
    pub session_id: String,
    pub from: u16,
    pub to: Option<u16>, // None 表示广播
    pub round: u8,
    pub payload: String,
}

type MessageStore = Arc<RwLock<HashMap<String, HashMap<u8, Vec<DkgMessage>>>>>;

#[tokio::main]
async fn main() {
    let store: MessageStore = Arc::new(RwLock::new(HashMap::new()));

    let app = Router::new()
        .route("/send", post(send_message))
        .route("/fetch/:session/:round/:id", get(fetch_messages))
        .with_state(store);

    println!("Relay running on http://localhost:9000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:9000").await.unwrap();
    serve(listener, app).await.unwrap();
}

async fn send_message(
    axum::extract::State(store): axum::extract::State<MessageStore>,
    Json(msg): Json<DkgMessage>,
) -> Json<&'static str> {
    let mut map = store.write().await;
    let session = map.entry(msg.session_id.clone()).or_default();
    session.entry(msg.round).or_default().push(msg);
    Json("ok")
}

async fn fetch_messages(
    axum::extract::State(store): axum::extract::State<MessageStore>,
    Path((session_id, round, id)): Path<(String, u8, u16)>,
) -> Json<Vec<DkgMessage>> {
    let map = store.read().await;
    let msgs = map
        .get(&session_id)
        .and_then(|rounds| rounds.get(&round))
        .map(|v| {
            v.iter()
                .filter(|m| m.to.is_none() || m.to == Some(id))
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    Json(msgs)
} 