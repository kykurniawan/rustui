use std::{io, time::Duration};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph, List, ListItem},
    Frame, Terminal,
};

struct App {
    messages: Vec<Spans<'static>>,
    input: String,
    contacts: Vec<&'static str>,
    selected_contact: usize,
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
                Span::raw(" Connection established to SECURE_NODE_772"),
            ]),
            Spans::from(vec![
                Span::raw("["),
                Span::styled("00:00:02", Style::default().fg(Color::DarkGray)),
                Span::raw("] "),
                Span::styled("system", Style::default().fg(Color::Cyan).add_modifier(tui::style::Modifier::BOLD)),
                Span::raw(":"),
                Span::raw(" Encryption: AES-256 | Protocol: TLS 1.3"),
            ]),
            Spans::from(vec![
                Span::raw("["),
                Span::styled("00:00:05", Style::default().fg(Color::DarkGray)),
                Span::raw("] "),
                Span::styled("shadow_runner", Style::default().fg(Color::Green).add_modifier(tui::style::Modifier::BOLD)),
                Span::raw(":"),
                Span::raw(" Hey, you got the access codes?"),
            ]),
            Spans::from(vec![
                Span::raw("["),
                Span::styled("00:00:12", Style::default().fg(Color::DarkGray)),
                Span::raw("] "),
                Span::styled("ghost", Style::default().fg(Color::Yellow).add_modifier(tui::style::Modifier::BOLD)),
                Span::raw(":"),
                Span::raw(" Yeah, intercepted the packet. DMZ is compromised."),
            ]),
            Spans::from(vec![
                Span::raw("["),
                Span::raw("00:00:18"),
                Span::raw("] "),
                Span::styled("shadow_runner", Style::default().fg(Color::Green).add_modifier(tui::style::Modifier::BOLD)),
                Span::raw(":"),
                Span::raw("Nice work. Meeting at 0300?"),
            ]),
            Spans::from(vec![
                Span::raw("["),
                Span::styled("00:00:25", Style::default().fg(Color::DarkGray)),
                Span::raw("] "),
                Span::styled("ghost", Style::default().fg(Color::Yellow).add_modifier(tui::style::Modifier::BOLD)),
                Span::raw(":"),
                Span::raw(" Copy that. Use the backup channel."),
            ]),
            Spans::from(vec![
                Span::raw("["),
                Span::styled("00:01:30", Style::default().fg(Color::DarkGray)),
                Span::raw("] "),
                Span::styled("system", Style::default().fg(Color::Cyan).add_modifier(tui::style::Modifier::BOLD)),
                Span::raw(":"),
                Span::raw(" >> Incoming transmission from NODE_X9"),
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
        }
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

    let header = Paragraph::new(
        Text::from(vec![Spans::from(vec![
            Span::raw(">> SECURE_CHAT v2.4.1 | "),
            Span::styled("ENCRYPTED", Style::default().fg(Color::Green).add_modifier(tui::style::Modifier::BOLD)),
            Span::raw(" | "),
            Span::styled("ONLINE", Style::default().fg(Color::Green)),
            Span::raw(" | channel: #darknet"),
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