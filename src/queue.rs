use crate::storage::{add_book, setup};
use crate::BookRequest;

pub fn run(
    BookRequest {
        id,
        title,
        auth,
        folder,
    }: BookRequest,
) -> Result<(), Box<dyn std::error::Error>> {
    setup()?;
    add_book(BookRequest {
        id,
        title,
        auth,
        folder,
    })
}
