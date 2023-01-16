use futures_executor::block_on;
use gitext::{handle, Result};

fn main() -> Result<()> {
    block_on(handle(std::env::args()))?;
    Ok(())
}
