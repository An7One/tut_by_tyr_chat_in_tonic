use anyhow::Result;
use tut_by_tyr_tonic_grpc::server;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    server::start().await;
    Ok(())
}
