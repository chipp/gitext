use error::Error;
use futures_executor::block_on;
use gitext::handle;

fn main() -> Result<(), Error> {
    block_on(handle(std::env::args()))
}
