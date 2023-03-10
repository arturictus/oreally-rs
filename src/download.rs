use crate::BookRequest;
use std::error;
use std::process::Command;

pub(crate) fn run(opts: BookRequest) -> Result<(), Box<dyn error::Error>> {
    let BookRequest {
        id,
        title,
        auth,
        folder,
    } = opts;
    let docker_command =
        format!("(docker run kirinnee/orly:latest login {id} {auth}) > \"{folder}/{title}.epub\"",);
    Command::new("sh").arg("-c").arg(docker_command).spawn()?;

    Ok(())
}
