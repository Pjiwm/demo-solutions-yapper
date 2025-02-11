use crate::server::Message;
use reqwest::blocking::Client;

pub fn send_message(address: &str, msg: Message) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let api_url = format!("http://{}/message", address);
    let res = client
        .post(api_url)
        .json(&msg)
        .send()?;

    println!("Response: {:?}", res.text()?);
    Ok(())
}
