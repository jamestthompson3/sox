use std::io::prelude::*;
use std::os::unix::net::UnixStream;

use crate::shared::spawn_job;

pub fn handle_client(job_id: String, cmd: String, cmd_args: Vec<String>) {
    let exit_code = spawn_job(cmd, cmd_args);
    let socket_path = format!("/tmp/sox-{}", job_id);
    let connection = UnixStream::connect(socket_path);
    match connection {
        Ok(mut socket) => {
            socket.write_all(exit_code.to_string().as_bytes()).unwrap();
        }
        Err(e) => println!("Listener job terminated with code: {:?}. Exiting now.", e),
    }
}
