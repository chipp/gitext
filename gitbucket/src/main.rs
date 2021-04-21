use futures_executor::block_on;
use gitbucket::*;

fn main() -> Result<(), String> {
    block_on(handle(std::env::args()))
}
