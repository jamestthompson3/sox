use std::io;
use std::io::prelude::*;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::thread;

use crate::shared::spawn_job;

use crossbeam_channel::{bounded, Receiver};
use signal_hook::{iterator::Signals, SIGINT};

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

pub fn listen_for(job_id: String, cmd: String, cmd_args: Vec<String>) {
    let interupt = signint_notifier().unwrap();
    let runner = socket_runner(job_id, cmd, cmd_args).unwrap();
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

fn socket_runner(job_id: String, cmd: String, cmd_args: Vec<String>) -> io::Result<Receiver<()>> {
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
                let mut msg = String::new();
                socket.read_to_string(&mut msg).unwrap();
                let status_code = msg.parse::<i32>().unwrap();
                if status_code == 0 {
                    spawn_job(cmd.to_owned(), cmd_args.to_owned());
                } else {
                    println!("Dependent job failed, aborting...");
                }
            }
            Err(err) => println!("Failed to connect: {:?}", err),
        };
    });
    Ok(r)
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
