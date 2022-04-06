mod discord;
mod mirror;

use dotenv::dotenv;
use std::env;

async fn say_hello() {
    println!("Hello");
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("USER_DISCORD_TOKEN").expect("Bot token not present");
    let t_id = env::var("TARGET_SERVER_ID").expect("Target server ID not present");
    let mirror_client = mirror::MirrorClient {
        token: token,
        server_id: t_id,
    };

    mirror_client.get_channels().await;

    /*
    let mut client = discord::create_client(token.to_string()).await; 

    if let Err(why) = client.start().await {
        println!("Error starting bot: {:?}", why);
    }
    */
}
