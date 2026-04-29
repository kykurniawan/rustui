use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use rustui::{App, Spans, draw_chat_screen};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

fn generate_id() -> String {
    let mut rng = rand::thread_rng();
    (0..6).map(|_| {
        let idx = rng.gen_range(0..36);
        if idx < 10 { (b'0' + idx) as char } else { (b'a' + idx - 10) as char }
    }).collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let my_id = generate_id();
    println!("Your ID: {}", my_id);

    let url = "ws://127.0.0.1:8080";
    let (ws_stream, _) = connect_async(url).await?;
    println!("Connected to server: {}", url);

    let (mut write, mut read) = ws_stream.split();

    let register_msg = serde_json::json!({
        "Register": { "id": &my_id }
    });
    write.send(Message::Text(register_msg.to_string())).await?;

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = tui::backend::CrosstermBackend::new(stdout);
    let mut terminal = tui::Terminal::new(backend)?;

    let mut app = App::new(my_id.clone());

    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);

    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                let _ = tx.send(text).await;
            }
        }
    });

    loop {
        terminal.draw(|f| {
            let size = f.size();
            draw_chat_screen(f, size, &mut app);
        })?;

        if let Some(msg) = rx.try_recv().ok() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&msg) {
                let msg_type = json.get("type").and_then(|v| v.as_str()).unwrap_or("");
                match msg_type {
                    "message" => {
                        let from = json.get("from").and_then(|v| v.as_str()).unwrap_or("unknown");
                        let text = json.get("msg").and_then(|v| v.as_str()).unwrap_or("");
                        app.add_message(format!(
                            "[{}] {}: {}",
                            "00:00:00",
                            from,
                            text
                        ));
                    }
                    "registered" => {
                        if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
                            if id != app.my_id {
                                app.add_contact(id.to_string(), id.to_string());
                            }
                        }
                    }
                    "list" => {
                        if let Some(clients) = json.get("clients").and_then(|v| v.as_array()) {
                            for client in clients {
                                if let Some(id) = client.as_str() {
                                    if id != app.my_id && !app.contacts.contains_key(id) {
                                        app.add_contact(id.to_string(), id.to_string());
                                    }
                                }
                            }
                        }
                    }
                    "error" => {
                        let err_msg = json.get("msg").and_then(|v| v.as_str()).unwrap_or("Unknown error");
                        app.add_message(format!("[system] Error: {}", err_msg));
                    }
                    _ => {}
                }
            } else {
                app.add_message(msg);
            }
        }

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Enter => {
                            if !app.input.is_empty() {
                                let input = app.input.trim().to_string();
                                
                                if input.starts_with("/add ") {
                                    let parts: Vec<&str> = input.splitn(2, ' ').collect();
                                    if parts.len() == 2 {
                                        let contact_id = parts[1].trim().to_string();
                                        if !contact_id.is_empty() {
                                            if app.contacts.contains_key(&contact_id) {
                                                app.add_message(format!("[system] Contact {} already exists", contact_id));
                                            } else {
                                                app.add_contact(contact_id.clone(), contact_id.clone());
                                                app.add_message(format!("[system] Added contact: {}", contact_id));
                                                
                                                let send_msg = serde_json::json!({
                                                    "SendTo": {
                                                        "to": contact_id.clone(),
                                                        "msg": "Hello!"
                                                    }
                                                });
                                                let _ = write.send(Message::Text(send_msg.to_string())).await;
                                            }
                                        }
                                    } else {
                                        app.add_message("[system] Usage: /add <id>".to_string());
                                    }
                                } else if input.starts_with("/open ") {
                                    let parts: Vec<&str> = input.splitn(2, ' ').collect();
                                    if parts.len() == 2 {
                                        let target_id = parts[1].trim().to_string();
                                        if app.contacts.contains_key(&target_id) {
                                            app.selected_contact = Some(target_id.clone());
                                            app.add_message(format!("[system] Opened chat with {}", target_id));
                                        } else {
                                            app.add_message(format!("[system] Contact {} not found", target_id));
                                        }
                                    } else {
                                        app.add_message("[system] Usage: /open <id>".to_string());
                                    }
                                } else if let Some(target_id) = &app.selected_contact {
                                    let msg = format!("[{}] you: {}", "00:02:00", input);
                                    app.messages.push(Spans::from(msg));
                                    
                                    let send_msg = serde_json::json!({
                                        "SendTo": {
                                            "to": target_id,
                                            "msg": input
                                        }
                                    });
                                    let _ = write.send(Message::Text(send_msg.to_string())).await;
                                } else {
                                    app.add_message("[system] Select a contact with /open <id>".to_string());
                                }
                                app.input.clear();
                            }
                        }
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}