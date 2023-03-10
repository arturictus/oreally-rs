use crate::storage::{add_book, setup};
use crate::BookRequest;

pub fn run(req: BookRequest) -> Result<(), Box<dyn std::error::Error>> {
    setup()?;
    add_book(req)
}
