use anyhow::Result;
use gitext::handle;

#[tokio::main]
async fn main() -> Result<()> {
    handle(std::env::args()).await?;
    Ok(())
}
