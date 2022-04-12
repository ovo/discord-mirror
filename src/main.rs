mod gateway;
mod mirror;

use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let user_token = env::var("USER_DISCORD_TOKEN").expect("Expected user token in env");
    let bot_token = env::var("BOT_DISCORD_TOKEN").expect("Expected bot token in env");
    let t_id = env::var("TARGET_SERVER_ID").expect("Expected target id in env");


    gateway::create_connection();

}
