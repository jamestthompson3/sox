use std::env;
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

pub fn spawn_job(cmd: String, mut cmd_args: Vec<String>) -> i32 {
    let path = env::current_dir().unwrap();
    // only run one command -- ignore shell chaining (for now)
    if cmd_args.len() > 1 {
        let index = cmd_args.iter().position(|arg| arg == "&&").unwrap();
        if index > 0 {
            cmd_args.resize(index, "".to_string());
        }
    }
    let status;
    // automatically run shell scripts when given .sh extension
    if cmd.ends_with(".sh") {
        cmd_args.insert(0, cmd.clone());
        status = Command::new("sh")
            .args(cmd_args)
            .current_dir(path)
            .status()
            .expect(&format!("Failed to start {}", &cmd));
    } else {
        status = Command::new(&cmd)
            .args(cmd_args)
            .current_dir(path)
            .status()
            .expect(&format!("Failed to start {}", &cmd));
    }
    status.code().unwrap()
}
