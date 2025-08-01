use clap::Parser;
use tokio::time::{sleep, Duration};
use std::sync::{Arc, Mutex};
use base64::Engine;

mod dkg;
mod network;
mod types;

use dkg::*;
use network::*;
use types::*;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    id: u16,
    #[arg(short, long)]
    relay: String,
    #[arg(short, long)]
    total: usize,
    #[arg(short, long)]
    session: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let state = Arc::new(Mutex::new(DkgState::new(args.id, 2, args.total)));

    println!("Node {} starting DKG session: {}", args.id, args.session);



    // Round 1: 发送广播消息
    {
        let mut s = state.lock().unwrap();
        let msg_bytes = s.execute_round1();
        drop(s);

        let msg = DkgMessage {
            session_id: args.session.clone(),
            from: args.id,
            to: None,
            round: 1,
            payload: base64::engine::general_purpose::STANDARD.encode(&msg_bytes),
        };
        send_to_relay(&args.relay, &msg).await;
        println!("Node {}: Round 1 message sent", args.id);
    }

    // Round 1: 拉取其他节点消息
    loop {
        let msgs = fetch_from_relay(&args.relay, &args.session, 1, args.id).await;
        let mut s = state.lock().unwrap();
        for m in msgs {
            if m.from != args.id && !s.received_round1.contains_key(&m.from) {
                let data = base64::engine::general_purpose::STANDARD.decode(&m.payload).unwrap();
                s.handle_round1(m.from, data);
            }
        }

        if s.is_round1_complete(args.total) {
            println!("Node {}: Round 1 complete", args.id);
            break;
        }

        drop(s);
        sleep(Duration::from_secs(1)).await;
    }

    // Round 2: 每个节点向其他节点发送私密消息
    {
        let mut s = state.lock().unwrap();
        let messages = s.execute_round2(args.total);
        drop(s);

        for (to, msg_bytes) in messages {
            let msg = DkgMessage {
                session_id: args.session.clone(),
                from: args.id,
                to: Some(to),
                round: 2,
                payload: base64::engine::general_purpose::STANDARD.encode(&msg_bytes),
            };
            send_to_relay(&args.relay, &msg).await;
        }
        println!("Node {}: Round 2 messages sent", args.id);
    }

    // Round 2: 拉取私密消息
    loop {
        let msgs = fetch_from_relay(&args.relay, &args.session, 2, args.id).await;
        let mut s = state.lock().unwrap();
        for m in msgs {
            if m.from != args.id && !s.received_round2.contains_key(&m.from) {
                let data = base64::engine::general_purpose::STANDARD.decode(&m.payload).unwrap();
                s.handle_round2(m.from, data);
            }
        }

        if s.is_round2_complete(args.total) {
            println!("Node {}: Round 2 complete", args.id);
            break;
        }

        drop(s);
        sleep(Duration::from_secs(1)).await;
    }

    // Round 3: 广播消息
    {
        let mut s = state.lock().unwrap();
        let msg_bytes = s.execute_round3();
        drop(s);

        let msg = DkgMessage {
            session_id: args.session.clone(),
            from: args.id,
            to: None,
            round: 3,
            payload: base64::engine::general_purpose::STANDARD.encode(&msg_bytes),
        };
        send_to_relay(&args.relay, &msg).await;
        println!("Node {}: Round 3 message sent", args.id);
    }

    // Round 3: 拉取广播消息
    loop {
        let msgs = fetch_from_relay(&args.relay, &args.session, 3, args.id).await;
        let mut s = state.lock().unwrap();
        for m in msgs {
            if m.from != args.id && !s.received_round3.contains_key(&m.from) {
                let data = base64::engine::general_purpose::STANDARD.decode(&m.payload).unwrap();
                s.handle_round3(m.from, data);
            }
        }

        if s.is_round3_complete(args.total) {
            println!("Node {}: Round 3 complete", args.id);
            break;
        }

        drop(s);
        sleep(Duration::from_secs(1)).await;
    }

    // 输出结果
    {
        println!("🔍 节点 {} 开始生成一致密钥共享...", args.id);
        let key_share = {
            let mut s = state.lock().unwrap();
            s.finalize()
        };
        
        if let Some(key_share) = key_share {
            println!("🎉 节点 {} DKG 协议完成，获得一致密钥共享！", args.id);
            println!("🔑 密钥共享类型: {:?}", std::any::type_name_of_val(&key_share));
            println!("📊 公钥信息: {:?}", key_share.shared_public_key());
            println!("🔢 节点索引: {}", key_share.i);
            println!("👥 总节点数: {}", key_share.n());
            
            // 验证公钥一致性
            let public_key = key_share.shared_public_key();
            println!("🔍 节点 {} 公钥验证: {:?}", args.id, public_key);
        } else {
            println!("❌ 节点 {} DKG 协议失败", args.id);
        }
    }
    
    println!("Node {} DKG session completed", args.id);
} 