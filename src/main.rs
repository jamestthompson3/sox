use std::process;

mod caster;
mod listener;
mod shared;

#[macro_use]
extern crate crossbeam_channel;
extern crate clipboard;

use structopt::StructOpt;

use caster::handle_client;
use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use listener::listen_for;
use shared::SoxCommand;

fn main() {
    let cli_args = SoxCommand::from_args();
    println!("{:?}", cli_args);
    match cli_args {
        SoxCommand::Listen {
            job_id,
            cmd,
            cmd_args,
        } => {
            listen_for(job_id.unwrap(), cmd.unwrap(), cmd_args);
        }
        SoxCommand::Cast { cmd, cmd_args } => {
            let job_id = process::id();
            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
            ctx.set_contents(String::from(format!("sox listen -j {}", job_id)))
                .unwrap();
            println!("Running job: \x1b[38;5;169m{}\x1b[0m", job_id);
            println!("📋 copied to clipboard!");
            handle_client(job_id.to_string(), cmd.unwrap(), cmd_args);
        }
    }
}
