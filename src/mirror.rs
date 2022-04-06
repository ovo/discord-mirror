use std::collections::HashMap;
use reqwest::Client;
use reqwest::header::{
    HeaderMap,
    AUTHORIZATION,
    CONTENT_TYPE,
};

#[derive(Debug)]
pub struct MirrorClient {
   pub token: String,
   pub server_id: String
}

impl MirrorClient {
    pub async fn get_new_id(self: &Self) -> Result<(), Box<dyn std::error::Error>> {

        Ok(())
    }

    pub async fn get_channels(self: &Self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("https://discord.com/api/v8/guilds/{}/channels", self.server_id);
        let mut hmap = HeaderMap::new();

        hmap.insert(AUTHORIZATION, self.token.parse().unwrap());
        hmap.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        
        let mut res = Client::new().get(url)
            .headers(hmap)
            .send()
            .await?
            .text()
            .await?;

        println!("{}", res);


        Ok(())
    }
}
