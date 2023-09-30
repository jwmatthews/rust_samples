extern crate reqwest;
extern crate serde_json;

use std::error::Error;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    dotenv().ok();
    let api_token = dotenv::var("API_TOKEN").expect("API_TOKEN not set");

    let client = reqwest::Client::new();
    let text = "Generate a creative image from this text.";
    let request_url = "https://api.exampleserver.com/generate-image";
    let response = client.post(request_url)
        .header("Authorization", format!("Bearer {}", api_token))
        .json(&serde_json::json!({"text": text}))
        .send()
        .await?;

    if response.status().is_success() {
        let bytes = response.bytes().await?;
        std::fs::write("creative_image.png", &bytes)?;
        println!("Image saved successfully.");
    } else {
        eprintln!("Failed to generate image: {:?}", response.text().await?);
    }

    Ok(())
}
