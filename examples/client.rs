use anyhow::Result;
use std::env;
use tokio::io::{self, AsyncBufReadExt};
use tut_by_tyr_tonic_grpc::client::Client;

#[tokio::main]
async fn main() -> Result<()> {
    let username = env::var("USERNAME1")?;
    let mut client = Client::new(username).await;
    client.login().await?;
    client.get_messages().await?;
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    while let Ok(Some(msg)) = stdin.next_line().await {
        client.send_message("lobby", msg).await?;
    }
    Ok(())
}
