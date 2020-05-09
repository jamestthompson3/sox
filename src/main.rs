use std::env::args;
use std::io;
use std::io::prelude::*;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

#[macro_use]
extern crate crossbeam_channel;

use crossbeam_channel::{bounded, Receiver};
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
    println!("SENDING A TEST MESSAGE");
    socket.write_all(b"TESTING").unwrap();
}

fn handle_socket(socket: &mut UnixStream) {
    let mut msg = String::new();
    socket.read_to_string(&mut msg).unwrap();
    println!("RECEIVED: {}", msg);
}

fn signint_notifier() -> io::Result<Receiver<()>> {
    let (s, r) = bounded(1);
    let signals = Signals::new(&[SIGINT]).unwrap();
    thread::spawn(move || {
        for _ in signals.forever() {
            if s.send(()).is_err() {
                break;
            }
        }
    });
    Ok(r)
}

fn socket_runner() -> io::Result<Receiver<()>> {
    let (s, r) = bounded(1);
    let socket_path = Path::new("/tmp/mysock");
    let runner = DeleteOnDrop::bind(socket_path).unwrap();
    thread::spawn(move || loop {
        if s.send(()).is_err() {
            break;
        }
        let runner_result = runner.listener.accept();
        match runner_result {
            Ok((mut socket, addr)) => {
                println!("Got a client! {:?}", addr);
                handle_socket(&mut socket);
            }
            Err(err) => println!("Failed to connect: {:?}", err),
        };
    });
    Ok(r)
}

fn listen_for(job_id: String) {
    let interupt = signint_notifier().unwrap();
    let runner = socket_runner().unwrap();
    loop {
        select! {
            recv(runner) -> _ => {
                println!("Running job: {}",job_id);
            }
            recv(interupt) -> _ => {
                println!("Received interupt");
                break;
            }
        }
    }
}

fn main() {
    let arg_list: Vec<String> = args().collect();
    let action = &arg_list[1];
    println!("ACTION: {}", action);
    if action == "listen" {
        let listener = thread::spawn(move || {
            let job_id = &arg_list[2];
            listen_for(job_id.to_owned());
        });
        listener.join().unwrap();
    }
    // Test sending data over the socket.
    let caster = thread::spawn(|| loop {
        handle_client();
        thread::sleep(Duration::from_secs(3));
    });
    caster.join().unwrap();
}
