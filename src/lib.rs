pub use tui::text::Spans;
pub use tui::style::Style;

use std::collections::HashMap;

use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Color,
    text::{Span, Text},
    widgets::{Block, Borders, Paragraph, List, ListItem, Wrap},
    Frame,
};

pub struct App {
    pub my_id: String,
    pub messages: Vec<Spans<'static>>,
    pub input: String,
    pub input_scroll: u16,
    pub contacts: HashMap<String, String>,
    pub selected_contact: Option<String>,
}

impl App {
    pub fn new(my_id: String) -> Self {
        let messages = vec![
            Spans::from(vec![
                Span::raw("["),
                Span::styled("00:00:01", Style::default().fg(Color::DarkGray)),
                Span::raw("] "),
                Span::styled("system", Style::default().fg(Color::Cyan).add_modifier(tui::style::Modifier::BOLD)),
                Span::raw(":"),
                Span::raw(" Connected! Your ID: "),
                Span::styled(my_id.clone(), Style::default().fg(Color::Yellow).add_modifier(tui::style::Modifier::BOLD)),
            ]),
            Spans::from(vec![
                Span::raw("["),
                Span::styled("00:00:02", Style::default().fg(Color::DarkGray)),
                Span::raw("] "),
                Span::styled("system", Style::default().fg(Color::Cyan).add_modifier(tui::style::Modifier::BOLD)),
                Span::raw(":"),
                Span::raw(" Press 'a' to add a contact"),
            ]),
        ];

        Self {
            my_id,
            messages,
            input: String::new(),
            input_scroll: 0,
            contacts: HashMap::new(),
            selected_contact: None,
        }
    }

    pub fn add_message(&mut self, msg: String) {
        self.messages.push(Spans::from(msg));
    }

    pub fn add_contact(&mut self, id: String, name: String) {
        self.contacts.insert(id, name);
    }
}

pub fn draw_chat_screen<W: std::io::Write>(f: &mut Frame<CrosstermBackend<W>>, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(5),
        ])
        .split(area);

    let status = format!("CONNECTED | ID: {}", app.my_id);

    let header = Paragraph::new(
        Text::from(vec![Spans::from(vec![
            Span::raw(">> SECURE_CHAT | "),
            Span::styled(status, Style::default().fg(Color::Green).add_modifier(tui::style::Modifier::BOLD)),
            Span::raw(" | /add <id> | /open <id>"),
        ])])
    )
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)))
    .style(Style::default().fg(Color::White));
    f.render_widget(header, chunks[0]);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20),
            Constraint::Min(0),
        ])
        .split(chunks[1]);

    let contact_items: Vec<ListItem> = app
        .contacts
        .iter()
        .map(|(id, name)| {
            let is_selected = app.selected_contact.as_ref().map_or(false, |s| s == id);
            let style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(tui::style::Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let prefix = if is_selected { ">" } else { " " };
            ListItem::new(format!("{} {} ({})", prefix, name, id))
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
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });

    f.render_widget(message_area, main_chunks[1]);

    let input_block = Paragraph::new(app.input.as_str())
        .block(
            Block::default()
                .title(" MESSAGE >> ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))
        )
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .scroll((app.input_scroll, 0));

    f.render_widget(input_block, chunks[2]);

    let input_area = chunks[2];
    if input_area.width > 2 && input_area.height > 2 {
        let line_width = (input_area.width - 2) as usize;
        let input_len = app.input.len();
        let _total_lines = (input_len + line_width - 1) / line_width.max(1);
        let visible_lines = (input_area.height - 2) as usize;

        let current_line = if line_width > 0 { input_len / line_width } else { 0 };
        
        if current_line >= (app.input_scroll as usize + visible_lines) {
            app.input_scroll = ((current_line + 1).saturating_sub(visible_lines)) as u16;
        } else if current_line < app.input_scroll as usize && app.input_scroll > 0 {
            app.input_scroll = current_line as u16;
        }

        let cursor_col = if line_width > 0 { input_len % line_width } else { 0 };
        let cursor_row = current_line.saturating_sub(app.input_scroll as usize);
        let cursor_x = input_area.x + 1 + cursor_col as u16;
        let cursor_y = input_area.y + 1 + cursor_row as u16;
        
        f.set_cursor(cursor_x, cursor_y);
    }
}