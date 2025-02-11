mod server;
use std::sync::mpsc;
use std::thread;
use server::{listen_server, Message};

fn main() {
    let (tx, rx): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel();

    // Start the Actix web server in a separate thread
    thread::spawn(move || {
        if let Err(e) = listen_server(tx) {
            eprintln!("Server failed: {}", e);
        }
    });

    // Simulating message processing in another thread
    thread::spawn(move || {
        while let Ok(msg) = rx.recv() {
            println!("Received message from {}: {}", msg.username, msg.message);
        }
    });

    // Ratatui UI here
    loop {}
}

