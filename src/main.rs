use gitext::{handle, Result};

#[tokio::main]
async fn main() -> Result<()> {
    handle(std::env::args()).await?;
    Ok(())
}
