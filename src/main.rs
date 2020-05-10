use std::io;
use std::process;
use std::thread;

mod caster;
mod listener;

use caster::handle_client;
use listener::socket_runner;

#[macro_use]
extern crate crossbeam_channel;
extern crate structopt;

use crossbeam_channel::{bounded, Receiver};
use signal_hook::{iterator::Signals, SIGINT};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "sox", about = "inter-process job signalling")]
struct Args {
    action: Actions,
    #[structopt(name = "job", required_if("action", "LISTEN"), short)]
    job_id: Option<String>,
    #[structopt(name = "command", required_if("action", "CAST"), short)]
    cmd: Option<String>,
    #[structopt(required_if("action", "CAST"), default_value = "")]
    cmd_args: Vec<String>,
}

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

impl ToString for Actions {
    fn to_string(&self) -> String {
        match self {
            Actions::CAST => String::from("cast"),
            Actions::LISTEN => String::from("listen"),
        }
    }
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
    // let arg_list: Vec<String> = args().collect();
    // let action: Actions = arg_list[1].parse().unwrap();
    let cli_args = Args::from_args();
    println!("{:?}", cli_args);
    match cli_args.action {
        Actions::LISTEN => {
            let listener = thread::spawn(move || {
                let job_id = cli_args.job_id;
                listen_for(job_id.unwrap());
            });
            listener.join().unwrap();
        }
        Actions::CAST => {
            let job_id = process::id();
            println!("Running job: \x1b[38;5;169m{}\x1b[0m", job_id);
            let caster = thread::spawn(move || {
                handle_client(job_id.to_string(), cli_args.cmd.unwrap(), cli_args.cmd_args);
            });
            caster.join().unwrap();
        }
    }
}
