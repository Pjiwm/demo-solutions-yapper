use std::time::Duration;
use crate::server::Message;
use reqwest::blocking::Client;

pub fn send_message(address: &str, msg: Message) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let api_url = format!("http://{}/message", address);
    client
        .post(api_url)
        .json(&msg)
        .timeout(Duration::from_millis(800))
        .send()?;
    Ok(())
}
