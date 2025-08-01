use crate::types::DkgMessage;
use reqwest::Client;

pub async fn send_to_relay(relay_url: &str, msg: &DkgMessage) {
    let client = Client::new();
    let _ = client.post(format!("{}/send", relay_url))
        .json(msg)
        .send()
        .await;
}

pub async fn fetch_from_relay(relay_url: &str, session_id: &str, round: u8, id: u16) -> Vec<DkgMessage> {
    let client = Client::new();
    match client
        .get(format!("{}/fetch/{}/{}/{}", relay_url, session_id, round, id))
        .send()
        .await
    {
        Ok(response) => {
            match response.json::<Vec<DkgMessage>>().await {
                Ok(messages) => messages,
                Err(_) => Vec::new(),
            }
        }
        Err(_) => Vec::new(),
    }
} 