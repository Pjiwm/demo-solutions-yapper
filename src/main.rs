mod server;
mod client;
mod tui;
use std::io::Write;
use std::sync::mpsc;
use std::{env, thread};
use server::{listen_server, Message};

fn main() {
    let (tx, rx): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel();
    let port = get_port_from_args();
    thread::spawn(move || {
        if let Err(e) = listen_server(tx, port) {
            eprintln!("Server failed: {}", e);
        }
    });
    let mut app = tui::App::new("Bruh".to_string());
    if let Err(err) = app.run(rx) {
        println!("{err}");
    }
}

/// Parses the command-line arguments and extracts a valid port number.
/// Defaults to 8080 if no valid port is provided.
fn get_port_from_args() -> usize {
    env::args()
        .collect::<Vec<_>>() 
        .windows(2) 
        .find_map(|args| {
            if args[0] == "--port" {
                args.get(1)?.parse::<usize>().ok()
            } else {
                None
            }
        })
        .filter(|&port| (1024..=65535).contains(&port)) 
        .unwrap_or_else(|| {
            eprintln!("Invalid or missing port. Using default: 8080");
            8080
        })
}

fn user_input_loop() {
    loop {
        print!("Enter target IP:PORT (or 'exit' to quit): ");
        std::io::stdout().flush().unwrap();
        
        let mut address = String::new();
        std::io::stdin().read_line(&mut address).unwrap();
        let address = address.trim();
        if address.eq_ignore_ascii_case("exit") {
            break;
        }

        print!("Enter your username: ");
        std::io::stdout().flush().unwrap();
        let mut username = String::new();
        std::io::stdin().read_line(&mut username).unwrap();
        
        print!("Enter your message: ");
        std::io::stdout().flush().unwrap();
        let mut message = String::new();
        std::io::stdin().read_line(&mut message).unwrap();

        let msg = Message {
            username: username.trim().to_string(),
            message: message.trim().to_string(),
        };

        if let Err(e) = client::send_message(address, msg) {
            eprintln!("❌ Failed to send message: {}", e);
        }
    }
}

fn spawn_message_listener(rx: mpsc::Receiver<Message>) {
    thread::spawn(move || {
        for msg in rx {
            println!("\n📩 New message from {}: {}\n", msg.username, msg.message);
        }
    });
}
