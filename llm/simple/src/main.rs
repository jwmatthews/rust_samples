use openai_rust;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let client = openai_rust::Client::new(&std::env::var("OPENAI_API_KEY").unwrap());
    let args = openai_rust::chat::ChatArguments::new("gpt-3.5-turbo", vec![
        openai_rust::chat::Message {
            role: "user".to_owned(),
            content: "Hello GPT!  Please tell me a random poem that talks about dobermanns and schutzhund.".to_owned(),
        },
    ]);
    let res = client.create_chat(args).await.unwrap();
    println!("{}", res);
}
