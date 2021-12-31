use futures_executor::block_on;
use gitext::handle;
use gitext::Error;

fn main() -> Result<(), Error> {
    block_on(handle(std::env::args()))
}
