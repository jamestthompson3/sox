use std::io;
use std::io::prelude::*;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::thread;

use crossbeam_channel::{bounded, Receiver};

pub fn socket_runner(job_id: String) -> io::Result<Receiver<()>> {
    let (s, r) = bounded(1);
    let socket_name = format!("/tmp/sox-{}", job_id);
    let socket_path = Path::new(&socket_name);
    let runner = DeleteOnDrop::bind(socket_path).unwrap();
    thread::spawn(move || loop {
        if s.send(()).is_err() {
            break;
        }
        let runner_result = runner.listener.accept();
        match runner_result {
            Ok((mut socket, _)) => {
                handle_socket(&mut socket);
            }
            Err(err) => println!("Failed to connect: {:?}", err),
        };
    });
    Ok(r)
}

fn handle_socket(socket: &mut UnixStream) {
    let mut msg = String::new();
    socket.read_to_string(&mut msg).unwrap();
    let status_code = msg.parse::<i32>().unwrap();
    if status_code == 0 {
        println!("Executing program!");
    } else {
        println!("Dependent job failed, aborting...");
    }
}

struct DeleteOnDrop {
    path: PathBuf,
    listener: UnixListener,
}

impl DeleteOnDrop {
    fn bind(path: impl AsRef<Path>) -> std::io::Result<Self> {
        // remove socket if already exists
        match std::fs::remove_file(&path) {
            Ok(_) => {}
            Err(_) => {}
        };
        let path = path.as_ref().to_owned();
        UnixListener::bind(&path).map(|listener| DeleteOnDrop { path, listener })
    }
}

impl Drop for DeleteOnDrop {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path).unwrap();
    }
}
