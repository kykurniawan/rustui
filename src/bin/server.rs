use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;

type ClientMap = Arc<RwLock<HashMap<String, mpsc::Sender<String>>>>;

#[derive(serde::Serialize, serde::Deserialize)]
enum Cmd {
    Register { id: String },
    Broadcast { msg: String },
    List,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let clients: ClientMap = Arc::new(RwLock::new(HashMap::new()));

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("WebSocket server listening on ws://127.0.0.1:8080");

    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                println!("New connection from: {}", peer_addr);
                let clients = clients.clone();

                tokio::spawn(async move {
                    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
                        Ok(s) => s,
                        Err(e) => {
                            println!("Error accepting connection: {}", e);
                            return;
                        }
                    };

                    let (mut write, mut read) = ws_stream.split();
                    let mut my_id = String::new();
                    let peer_str = peer_addr.to_string();

                    let (tx, mut rx) = mpsc::channel::<String>(100);
                    {
                        let mut clients = clients.write().await;
                        clients.insert(peer_str.clone(), tx);
                    }

                    loop {
                        tokio::select! {
                            msg = rx.recv() => {
                                if let Some(text) = msg {
                                    if write.send(Message::Text(text)).await.is_err() {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }
                            msg = read.next() => {
                                match msg {
                                    Some(Ok(Message::Text(text))) => {
                                        if let Ok(cmd) = serde_json::from_str::<Cmd>(&text) {
                                            match cmd {
                                                Cmd::Register { id } => {
                                                    let mut clients = clients.write().await;
                                                    clients.remove(&peer_str);
                                                    if let Some(old_tx) = clients.remove(&id) {
                                                        drop(old_tx);
                                                    }
                                                    my_id = id.clone();
                                                    let (new_tx, new_rx) = mpsc::channel::<String>(100);
                                                    clients.insert(id.clone(), new_tx);

                                                    let ids: Vec<String> = clients.keys().cloned().collect();
                                                    
                                                    let list_msg = serde_json::json!({"type": "list", "clients": ids.clone()}).to_string();
                                                    
                                                    for (_, sender) in clients.iter() {
                                                        sender.send(list_msg.clone()).await.ok();
                                                    }
                                                    
                                                    write.send(Message::Text(
                                                        serde_json::json!({"type": "registered", "id": id}).to_string()
                                                    )).await.ok();
                                                    
                                                    write.send(Message::Text(
                                                        serde_json::json!({"type": "list", "clients": ids}).to_string()
                                                    )).await.ok();
                                                    
                                                    println!("Client registered: {}", id);
                                                    
                                                    rx = new_rx;
                                                }
                                                Cmd::Broadcast { msg } => {
                                                    let from = if my_id.is_empty() { peer_str.clone() } else { my_id.clone() };
                                                    let clients = clients.read().await;
                                                    let payload = serde_json::json!({
                                                        "type": "message",
                                                        "from": from,
                                                        "msg": msg
                                                    }).to_string();
                                                    for (id, sender) in clients.iter() {
                                                        if id != &my_id {
                                                            let _ = sender.send(payload.clone()).await;
                                                        }
                                                    }
                                                    println!("Broadcast from {}: {}", from, msg);
                                                }
                                                Cmd::List => {
                                                    let clients = clients.read().await;
                                                    let ids: Vec<String> = clients.keys().cloned().collect();
                                                    write.send(Message::Text(
                                                        serde_json::json!({"type": "list", "clients": ids}).to_string()
                                                    )).await.ok();
                                                }
                                            }
                                        }
                                    }
                                    Some(Ok(Message::Close(_))) => {
                                        println!("Client {} disconnected", peer_addr);
                                        let mut clients = clients.write().await;
                                        if !my_id.is_empty() {
                                            clients.remove(&my_id);
                                        } else {
                                            clients.remove(&peer_str);
                                        }
                                        break;
                                    }
                                    None | Some(Err(_)) => break,
                                    _ => {}
                                }
                            }
                        }
                    }
                });
            }
            Err(e) => {
                println!("Error accepting connection: {}", e);
            }
        }
    }
}