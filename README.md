# Workshop
Deze demo laat zien hoe je een eenvoudige Message Sender applicatie bouwt,
bestaande uit een server en een client,
waarbij je berichten kunt verzenden naar andere gebruikers via een IP-adres en poort.
Beide componenten (server en client) worden zelf geschreven in Rust.
De applicatie maakt gebruik van een Terminal User Interface (TUI)
die al is gegeven,
waarmee de gebruiker berichten kan versturen en ontvangen in de terminal.
Probeer zo veel mogelijk zelf te typen! Geen copy-paste.

## Deel 1: Web server

### Stap 0: Voeg de afhankelijkheden toe aan `Cargo.toml`
Open het bestand Cargo.toml en voeg de volgende afhankelijkheden toe:
```toml
[dependencies]
actix-web = "4"
serde = "1"
serde_derive = "1"
serde_json = "1"

```

### Stap 1: Maak het Message struct
Open de file `server.rs`
Dit is waar we de logica van onze server zullen plaatsen.
Begin met het definiëren van het Message struct.
Dit struct bevat twee velden: username en message, die we van de client zullen ontvangen.
Achter de puntjes volgen de missende fields. Bij derive missen: Serialize, Deserialize en Clone.
Dit zijn 'interfaces' die het Message struct verstuurbaar maken.

```rs
use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use serde_derive::{Deserialize, Serialize};
use std::sync::mpsc::Sender;

// Stap 1: Definieer het Message struct
#[derive(Debug, ...)]
pub struct Message {
    pub username: String,
    ...
}
```

### Stap 2: Maak een handler voor het ontvangen van berichten
We moeten nu een handler maken die de berichten ontvangt via een POST-request.
Deze functie verwerkt de ontvangen JSON-gegevens,
stuurt ze door via een Sender en stuurt een bevestigingsbericht terug.
```rs
#[post("/message")]
async fn receive_message(
    msg: web::Json<Message>,                 // De ontvangen JSON van de client
    sender: web::Data<Sender<Message>>,      // De sender om berichten door te sturen
) -> impl Responder {
    // Probeer het bericht door te sturen naar de sender
    if let Err(e) = sender.send(msg.into_inner()) {
        eprintln!("Failed to send message: {}", e);  // Print een fout als het versturen mislukt
    }

    // Bevestiging naar de client
    HttpResponse::Ok().body("Message received")
}
```

### Stap 3: Bouw de server
Nu gaan we de server bouwen die luistert naar inkomende verbindingen.
De server maakt gebruik van de HttpServer van Actix-web 
en maakt een route aan voor de POST-request naar /message.
```rs
pub fn listen_server(tx: Sender<Message>, port: usize) -> std::io::Result<()> {
    // De serveradres en poort
    let address = format!("0.0.0.0:{}", port);
    
    // Bouw de server
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tx.clone()))  // Deel de sender tussen verschillende threads
            .service(receive_message)              // Voeg de message-handler toe aan de server
    })
    .bind(&address)?                            // Bind de server aan het adres
    .run();
    
    // Start de server
    actix_web::rt::System::new().block_on(server)
}
```
### Stap 4: Start de server in een apart thread
In `main.rs` starten we de server in een nieuw thread met thread::spawn.
Dit zorgt ervoor dat de server onafhankelijk van de hoofdthread kan draaien.
In de thread wordt de listen_server functie aangeroepen 
met de tx (sender) en de poort die uit de argumenten komt.
```rs
    let (tx, rx): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel();
    let port = get_port_from_args();
    let tx_clone = tx.clone();
    thread::spawn(move || {
        if let Err(e) = listen_server(tx_clone, port) {
            eprintln!("Server failed: {}", e);
        }
    });
```

### Stap 5: Kies een gebruikersnaam
```rs
    let mut app = tui::App::new("Ferris".to_string(), format!("{}:{}", ip_address, port));
```

## Deel 1: Client

### Stap 6: Verstuur een bericht naar de server
Open het bestand Cargo.toml en voeg de volgende afhankelijkheden toe:
```toml
reqwest = { version = "0.12", features = ["json", "blocking"] }
```
Maak de functie send_message

We maken de functie send_message die een POST-aanroep naar de server verstuurt.
De functie neemt twee argumenten:
```
address: Het adres van de server (bijvoorbeeld "localhost:8080").
msg: Het bericht dat we willen versturen, van het type Message.
```

```rs
use crate::server::Message;
use reqwest::blocking::Client;

pub fn send_message(address: &str, msg: Message) -> Result<(), reqwest::Error> {
    // Maak een nieuwe HTTP-client
    let client = Client::new();

    // Stel de URL in waar het bericht naartoe moet worden gestuurd
    let api_url = format!("http://{}/message", address);

    // Verstuur het POST-verzoek met het bericht als JSON
    let res = client
        .post(api_url)                       // POST-aanroep naar de server
        .json(&msg)                          // Zet het bericht om naar JSON
        .timeout(Duration::from_millis(800)) // Timeout na 800ms.
        .send()?;                            // Verstuur het verzoek

    // Print de response van de server
    println!("Response: {:?}", res.text()?);

    // Geef Ok terug om aan te geven dat de functie succesvol was
    Ok(())
}
```

### Stap 7: Draai de app
```
cargo run -- --port 8080
```
(Port kan zelf gekozen worden)
