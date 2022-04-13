mod gateway;
mod mirror;

use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();

    env::var("USER_DISCORD_TOKEN").expect("Expected user token in env");
    env::var("BOT_DISCORD_TOKEN").expect("Expected bot token in env");
    env::var("TARGET_SERVER_ID").expect("Expected target id in env");

    let client = gateway::create_client()
        .map_err(|e| eprintln!("Error connecting: {}", e))
        .unwrap();

    gateway::listen(client);

}
