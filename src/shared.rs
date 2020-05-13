use std::process::Command;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "sox", about = "inter-process job signalling")]
pub enum SoxCommand {
    #[structopt(name = "cast")]
    Cast {
        #[structopt(name = "command", short)]
        cmd: Option<String>,
        #[structopt(default_value = "")]
        cmd_args: Vec<String>,
    },
    #[structopt(name = "listen")]
    Listen {
        #[structopt(name = "job", short)]
        job_id: Option<String>,
        #[structopt(name = "command", short)]
        cmd: Option<String>,
        #[structopt(default_value = "")]
        cmd_args: Vec<String>,
    },
}

pub fn spawn_job(cmd: String, cmd_args: Vec<String>) -> i32 {
    let status = Command::new(&cmd)
        .args(cmd_args)
        .status()
        .expect(&format!("Failed to start {}", &cmd));
    status.code().unwrap()
}
