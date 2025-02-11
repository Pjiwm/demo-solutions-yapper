use crate::server::Message;
use crossterm::{
    event::{self, KeyCode, KeyEvent},
    execute, terminal,
};
use ratatui::{prelude::*, widgets::*};
use std::{io, sync::mpsc::Receiver};

pub struct App {
    pub username: String,
    pub input: String,
    pub address: String,
    pub messages: Vec<Message>,
    pub focus: Focus,
}

#[derive(Debug, PartialEq)]
pub enum Focus {
    Address,
    Input,
}

impl App {
    pub fn new(username: String) -> Self {
        Self {
            username,
            input: String::new(),
            address: String::new(),
            messages: Vec::new(),
            focus: Focus::Address, // Start with address input
        }
    }

    pub fn run(&mut self, rx: Receiver<Message>) -> io::Result<()> {
        let mut stdout = io::stdout();
        execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal::enable_raw_mode()?;
        loop {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let event::Event::Key(KeyEvent { code, .. }) = event::read()? {
                    match code {
                        KeyCode::Esc => break,
                        KeyCode::Tab => {
                            // Switch focus between address and input
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
                                let msg = Message {
                                    username: self.username.clone(),
                                    message: self.input.clone(),
                                };
                                if let Err(e) = crate::client::send_message(&self.address, msg) {
                                    eprintln!("Failed to send: {}", e);
                                }
                                self.input.clear();
                            }
                        }
                        KeyCode::Char(c) => {
                            // Input handling based on focus
                            match self.focus {
                                Focus::Address => self.address.push(c),
                                Focus::Input => self.input.push(c),
                            }
                        }
                        KeyCode::Backspace => match self.focus {
                            Focus::Address => {
                                self.address.pop();
                            }
                            Focus::Input => {
                                self.input.pop();
                            }
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
                Constraint::Length(3), // Address field
                Constraint::Length(3), // Input field
                Constraint::Min(0),    // Messages
            ])
            .split(f.area());

        let address_border = if self.focus == Focus::Address {
            Borders::ALL
        } else {
            Borders::ALL
        };

        let input_border = if self.focus == Focus::Input {
            Borders::ALL
        } else {
            Borders::ALL
        };

        let address = Paragraph::new(self.address.as_str()).block(
            Block::default()
                .borders(address_border)
                .title("Address (Tab to switch)"),
        );
        f.render_widget(address, chunks[0]);

        let input = Paragraph::new(self.input.as_str()).block(
            Block::default()
                .borders(input_border)
                .title("Input (Tab to switch)"),
        );
        f.render_widget(input, chunks[1]);

        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .map(|m| ListItem::new(format!("{}: {}", m.username, m.message)))
            .collect();
        let messages =
            List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
        f.render_widget(messages, chunks[2]);
    }
}
