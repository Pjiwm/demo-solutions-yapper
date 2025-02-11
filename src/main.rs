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
