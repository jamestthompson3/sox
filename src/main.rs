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

#[derive(Debug)]
enum Actions {
    LISTEN,
    CAST,
}

impl std::str::FromStr for Actions {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "listen" => Ok(Actions::LISTEN),
            "cast" => Ok(Actions::CAST),
            _ => Err(format!("{} is not a valid action", s)),
        }
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

fn handle_client(job_id: String) {
    let socket_path = format!("/tmp/sox-{}", job_id);
    let mut socket = UnixStream::connect(socket_path).unwrap();
    // TODO remove loop after testing
    loop {
        socket.write_all(b"TESTING").unwrap();
        thread::sleep(Duration::from_secs(3));
    }
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

fn socket_runner(job_id: String) -> io::Result<Receiver<()>> {
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

fn listen_for(job_id: String) {
    let interupt = signint_notifier().unwrap();
    let runner = socket_runner(job_id).unwrap();
    loop {
        select! {
            recv(runner) -> _ => {
            }
            recv(interupt) -> _ => {
                break;
            }
        }
    }
}

fn main() {
    let arg_list: Vec<String> = args().collect();
    let action: Actions = arg_list[1].parse().unwrap();
    match action {
        Actions::LISTEN => {
            let listener = thread::spawn(move || {
                let job_id = &arg_list[2];
                listen_for(job_id.to_owned());
            });
            listener.join().unwrap();
        }
        Actions::CAST => {
            let caster = thread::spawn(move || {
                let job_id = &arg_list[2];
                handle_client(job_id.to_owned());
            });
            caster.join().unwrap();
        }
    }
}
