use crate::server::Message;
use crossterm::{
    event::{self, KeyCode, KeyEvent},
    execute, terminal,
};
use ratatui::{
    prelude::*,
    widgets::*,
    style::{Style, Color},
};
use std::{io, sync::mpsc::Receiver, time::{Instant, Duration}};

pub struct App {
    pub username: String,
    pub input: String,
    pub address: String,
    pub ip_address: String,
    pub messages: Vec<Message>,
    pub focus: Focus,
    pub last_blink: Instant,
    pub blink_visible: bool,
}

#[derive(Debug, PartialEq)]
pub enum Focus {
    Address,
    Input,
}

impl App {
    pub fn new(username: String, ip_address: String) -> Self {
        Self {
            username,
            input: String::new(),
            address: String::new(),
            ip_address,
            messages: Vec::new(),
            focus: Focus::Address, // Start with address input
            last_blink: Instant::now(),
            blink_visible: true,
        }
    }

    pub fn run(&mut self, rx: Receiver<Message>, tx: std::sync::mpsc::Sender<Message>) -> io::Result<()> {
        let mut stdout = io::stdout();
        execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal::enable_raw_mode()?;
        loop {
            let now = Instant::now();
            if now.duration_since(self.last_blink) >= Duration::from_millis(500) {
                self.blink_visible = !self.blink_visible;
                self.last_blink = now;
            }

            terminal.draw(|f| self.ui(f))?;

            if event::poll(Duration::from_millis(100))? {
                if let event::Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                    match code {
                        KeyCode::Esc => break,
                        KeyCode::Char('c') if modifiers.contains(event::KeyModifiers::CONTROL) => break,
                        KeyCode::Tab => {
                            self.focus = match self.focus {
                                Focus::Address => Focus::Input,
                                Focus::Input => Focus::Address,
                            };
                        }
                        KeyCode::Enter => {
                            if self.focus == Focus::Input
                                && !self.input.is_empty()
                                && !self.address.is_empty()
                                {
                                    let address = self.address.clone();
                                    let msg = Message {
                                        username: self.username.clone(),
                                        message: self.input.clone(),
                                    };
                                    let tx_clone = tx.clone(); // Clone sender for thread use

                                    std::thread::spawn(move || {
                                        tx_clone.send(Message {
                                            username: "Server".to_string(),
                                            message: "Sending message...".to_string()
                                        }).expect("Error putting message on message list");
                                        match crate::client::send_message(&address, msg.clone()) {
                                            Ok(_) => {
                                                // Successfully sent message, send it to UI
                                                let _ = tx_clone.send(msg);
                                            }
                                            Err(e) => {
                                                // If there is an error, send an error message to UI
                                                let _ = tx_clone.send(Message {
                                                    username: "Server".to_string(),
                                                                      message: format!("Failed to send: {}", e),
                                                });
                                            }
                                        }
                                    });
                                }
                        }
                        KeyCode::Char(c) => match self.focus {
                            Focus::Address => self.address.push(c),
                            Focus::Input => self.input.push(c),
                        },
                        KeyCode::Backspace => match self.focus {
                            Focus::Address => { self.address.pop(); }
                            Focus::Input => { self.input.pop(); }
                        },
                        _ => {}
                    }
                }
            }

            while let Ok(msg) = rx.try_recv() {
                self.messages.push(msg);
            }
        }
        terminal::disable_raw_mode()?;
        Ok(())
    }

    fn ui(&self, f: &mut Frame) {
        let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // IP Address field
                     Constraint::Length(3), // Address field
                     Constraint::Length(3), // Input field
                     Constraint::Min(0),    // Messages
        ])
        .split(f.area());

        let cursor_symbol = if self.blink_visible { "|" } else { " " };

        let address_text = if self.focus == Focus::Address {
            format!("{}{}", self.address, cursor_symbol)
        } else {
            self.address.clone()
        };

        let input_text = if self.focus == Focus::Input {
            format!("{}{}", self.input, cursor_symbol)
        } else {
            self.input.clone()
        };

        let ip_display = Paragraph::new(self.ip_address.clone()).block(
            Block::default()
            .borders(Borders::ALL)
            .title("IP Address")
            .border_style(Style::default().fg(Color::Green)),
        );
        f.render_widget(ip_display, chunks[0]);

        let address = Paragraph::new(address_text).block(
            Block::default()
            .borders(Borders::ALL)
            .title("Address (Tab to switch)")
            .border_style(if self.focus == Focus::Address {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            }),
        );
        f.render_widget(address, chunks[1]);

        let input = Paragraph::new(input_text).block(
            Block::default()
            .borders(Borders::ALL)
            .title("Input (Tab to switch)")
            .border_style(if self.focus == Focus::Input {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            }),
        );
        f.render_widget(input, chunks[2]);

        let messages: Vec<ListItem> = self
        .messages
        .iter()
        .map(|m| ListItem::new(format!("{}: {}", m.username, m.message)))
        .collect();
        let messages =
        List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
        f.render_widget(messages, chunks[3]);
    }
}
