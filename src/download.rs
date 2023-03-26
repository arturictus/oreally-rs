use crate::BookRequest;
use std::error;
use std::process::{Command, Stdio};

pub(crate) fn run(request: BookRequest) -> Result<(), Box<dyn error::Error>> {
    let child = Command::new("sh")
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .arg("-c")
        .arg(request.to_cmd())
        .spawn()?;
    child.wait_with_output()?;

    Ok(())
}
