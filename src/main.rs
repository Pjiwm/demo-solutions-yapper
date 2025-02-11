mod server;
mod client;
mod tui;
use std::sync::mpsc;
use std::{env, thread};
use server::{listen_server, Message};
use local_ip_address::local_ip;

fn main() {
    let (tx, rx): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel();
    let port = get_port_from_args();
    let tx_clone = tx.clone();
    thread::spawn(move || {
        if let Err(e) = listen_server(tx_clone, port) {
            eprintln!("Server failed: {}", e);
        }
    });

    let ip_address = match local_ip() {
        Ok(ip) => ip.to_string(),
        Err(_) => "Unknown IP-address".to_string()
    };

    let mut app = tui::App::new("Bruh".to_string(), format!("{}:{}", ip_address, port));
    if let Err(err) = app.run(rx, tx) {
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