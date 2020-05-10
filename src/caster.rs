use std::io::prelude::*;
use std::os::unix::net::UnixStream;
use std::process::Command;

pub fn handle_client(job_id: String, cmd: String, cmd_args: Vec<String>) {
    let status = Command::new(&cmd)
        .args(cmd_args)
        .status()
        .expect(&format!("Failed to start {}", &cmd));
    let socket_path = format!("/tmp/sox-{}", job_id);
    let mut socket = UnixStream::connect(socket_path).unwrap();
    let exit_code = status.code().unwrap();
    socket.write_all(exit_code.to_string().as_bytes()).unwrap();
    // TODO remove loop after testing
}
