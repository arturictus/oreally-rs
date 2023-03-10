use crate::BookRequest;
use std::error;
use std::process::{Command, Stdio};

pub(crate) fn run(opts: BookRequest) -> Result<(), Box<dyn error::Error>> {
    let BookRequest {
        book_id,
        title,
        auth,
        folder,
    } = opts;
    let docker_command = format!(
        "(docker run kirinnee/orly:latest login {book_id} {auth}) > \"{folder}/{title}.epub\"",
    );
    let child = Command::new("sh")
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .arg("-c")
        .arg(docker_command)
        .spawn()?;
    child.wait_with_output()?;

    Ok(())
}
