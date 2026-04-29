use std::{io, time::Duration};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures_util::StreamExt;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph, List, ListItem},
    Frame, Terminal,
};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

struct App {
    messages: Vec<Spans<'static>>,
    input: String,
    contacts: Vec<&'static str>,
    selected_contact: usize,
    connected_clients: usize,
}

impl App {
    fn new() -> Self {
        let messages = vec![
            Spans::from(vec![
                Span::raw("["),
                Span::styled("00:00:01", Style::default().fg(Color::DarkGray)),
                Span::raw("] "),
                Span::styled("system", Style::default().fg(Color::Cyan).add_modifier(tui::style::Modifier::BOLD)),
                Span::raw(":"),
                Span::raw(" WebSocket server started on port 8080"),
            ]),
            Spans::from(vec![
                Span::raw("["),
                Span::styled("00:00:02", Style::default().fg(Color::DarkGray)),
                Span::raw("] "),
                Span::styled("system", Style::default().fg(Color::Cyan).add_modifier(tui::style::Modifier::BOLD)),
                Span::raw(":"),
                Span::raw(" Waiting for connections..."),
            ]),
        ];

        let contacts = vec![
            "shadow_runner",
            "ghost",
            "zero_cool",
            "acid_burn",
            "cerealkiller",
            "phantom",
        ];

        Self {
            messages,
            input: String::new(),
            contacts,
            selected_contact: 0,
            connected_clients: 0,
        }
    }

    fn add_message(&mut self, msg: String) {
        self.messages.push(Spans::from(msg));
    }
}

fn draw_chat_screen<W: std::io::Write>(f: &mut Frame<CrosstermBackend<W>>, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    let status = if app.connected_clients > 0 {
        format!("ONLINE | {} client(s) connected", app.connected_clients)
    } else {
        "WAITING".to_string()
    };

    let header = Paragraph::new(
        Text::from(vec![Spans::from(vec![
            Span::raw(">> SECURE_CHAT v2.4.1 | "),
            Span::styled(status, Style::default().fg(if app.connected_clients > 0 { Color::Green } else { Color::Yellow }).add_modifier(tui::style::Modifier::BOLD)),
            Span::raw(" | channel: #darknet | ws://localhost:8080"),
        ])])
    )
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)))
    .style(Style::default().fg(Color::White));
    f.render_widget(header, chunks[0]);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(18),
            Constraint::Min(0),
        ])
        .split(chunks[1]);

    let contact_items: Vec<ListItem> = app
        .contacts
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let style = if i == app.selected_contact {
                Style::default().fg(Color::Yellow).add_modifier(tui::style::Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let prefix = if i == app.selected_contact { ">" } else { " " };
            ListItem::new(format!("{} {}", prefix, name))
                .style(style)
        })
        .collect();

    let contact_list = List::new(contact_items)
        .block(Block::default().title(" CONTACTS ").borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)))
        .style(Style::default().fg(Color::White));

    f.render_widget(contact_list, main_chunks[0]);

    let message_area = Paragraph::new(Text::from(app.messages.clone()))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(message_area, main_chunks[1]);

    let input_block = Paragraph::new(app.input.as_str())
        .block(
            Block::default()
                .title(" MESSAGE >> ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))
        )
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Left);

    f.render_widget(input_block, chunks[2]);

    if f.size().height > 0 {
        let input_area = chunks[2];
        if input_area.width > 5 {
            let cursor_x = input_area.x + 1 + app.input.len() as u16;
            let cursor_y = input_area.y + 1;
            if cursor_x < input_area.right() {
                f.set_cursor(cursor_x, cursor_y);
            }
        }
    }
}

fn main() -> Result<(), io::Error> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let (tx, mut rx) = mpsc::channel::<String>(100);

    rt.spawn(async move {
        let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
        println!("WebSocket server listening on 127.0.0.1:8080");

        loop {
            tokio::select! {
                result = listener.accept() => {
                    if let Ok((stream, peer_addr)) = result {
                        println!("New connection from: {}", peer_addr);
                        let tx = tx.clone();

                        tokio::spawn(async move {
                            let ws_stream = match tokio_tungstenite::accept_async(stream).await {
                                Ok(s) => s,
                                Err(_) => return,
                            };
                            let (_, mut read) = ws_stream.split();

                            while let Some(msg) = read.next().await {
                                match msg {
                                    Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                                        let _ = tx.send(text).await;
                                    }
                                    Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => {
                                        println!("Client {} disconnected", peer_addr);
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        });
                    }
                }
            }
        }
    });

    enable_raw_mode()?;
    let mut stdout = io::stdout();

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| {
            let size = f.size();
            draw_chat_screen(f, size, &app);
        })?;

        while let Some(msg) = rx.try_recv().ok() {
            app.add_message(msg);
            app.connected_clients += 1;
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
                                let msg = format!(
                                    "[{}] you: {}",
                                    "00:02:00",
                                    app.input
                                );
                                app.messages.push(Spans::from(msg));
                                app.input.clear();
                            }
                        }
                        KeyCode::Up => {
                            if app.selected_contact > 0 {
                                app.selected_contact -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if app.selected_contact < app.contacts.len() - 1 {
                                app.selected_contact += 1;
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