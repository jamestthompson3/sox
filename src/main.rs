use std::io::prelude::*;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use crossbeam_channel::bounded;
use signal_hook::{iterator::Signals, SIGINT};

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

fn handle_client() {
    let mut socket = UnixStream::connect("/tmp/mysock").unwrap();
    socket.write_all(b"TESTING").unwrap();
}

fn handle_socket(socket: &mut UnixStream) {
    let mut msg = String::new();
    socket.read_to_string(&mut msg).unwrap();
    println!("RECEIVED: {}", msg);
}

fn main() {
    let (s, r) = bounded(1);
    let signals = Signals::new(&[SIGINT]).unwrap();
    thread::spawn(move || {
        for _ in signals.forever() {
            s.send("KILL").unwrap();
        }
    });
    let socket_path = Path::new("/tmp/mysock");
    let runner = DeleteOnDrop::bind(socket_path).unwrap();
    // Test sending data over the socket.
    thread::spawn(|| loop {
        handle_client();
        thread::sleep(Duration::from_secs(3));
    });
    loop {
        let msg = r.try_recv();
        match msg {
            Ok(sig) => {
                if sig == "KILL" {
                    break;
                }
            }
            _ => {
                match runner.listener.accept() {
                    Ok((mut socket, addr)) => {
                        println!("Got a client! {:?}", addr);
                        handle_socket(&mut socket);
                    }
                    Err(err) => println!("Failed to connect: {:?}", err),
                };
            }
        };
    }
}
