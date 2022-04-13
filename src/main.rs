mod gateway;
mod mirror;

use dotenv::dotenv;
use tokio::runtime::Builder;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let user_token = env::var("USER_DISCORD_TOKEN").expect("Expected user token in env");
    let server_id = env::var("TARGET_SERVER_ID").expect("Expected target id in env");
    env::var("BOT_DISCORD_TOKEN").expect("Expected bot token in env");

    let mut client = gateway::Gateway::new(user_token, server_id);
    client.listen();

}
