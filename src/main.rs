use anyhow::Result;
use iradio::app::run;

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}
