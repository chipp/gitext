use futures_executor::block_on;
use gitbucket::*;

fn main() -> Result<(), Error> {
    block_on(handle(std::env::args()))
}
