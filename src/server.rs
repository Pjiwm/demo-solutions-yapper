use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use serde_derive::{Deserialize, Serialize};
use std::sync::mpsc::Sender;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub username: String,
    pub message: String,
}

async fn receive_message(
    msg: web::Json<Message>,
    sender: web::Data<Sender<Message>>,
) -> impl Responder {
    if let Err(e) = sender.send(msg.into_inner()) {
        eprintln!("Failed to send message: {}", e);
    }
    HttpResponse::Ok().body("Message received")
}

pub fn listen_server(tx: Sender<Message>, port: usize) -> std::io::Result<()> {
    let address = format!("0.0.0.0:{}", port);
    
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tx.clone()))
            .route("/message", web::post().to(receive_message))
    })
    .bind(&address)?
    .run();
    
    actix_web::rt::System::new().block_on(server)
}
