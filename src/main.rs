extern crate users;
extern crate libc;

use std::process;
use std::env;

struct RunOptions {
    env: Option<Vec<(String, String)>>,
    user: Option<String>,
    group: Option<String>,
    work_dir: Option<String>,
    exit_timeout: Option<i64>
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let env: Vec<(String, String)> = env::vars().collect();
    println!("My pid is {}", process::id());
    println!("Args: {:?}", args);
    println!("Env: {:?}", env);

    let args = vec![String::from("-alh")];
    run(String::from("ls"), args, None);

    let args = vec![];
    run(String::from("env"), args, Some(env));
}

fn run(cmd: String,
       args: Vec<String>,
       env: Option<Vec<(String, String)>>) {
    let options: RunOptions = RunOptions {
        env: env,
        user: None,
        group: None,
        work_dir: None,
        exit_timeout: None
    };
    run_with_options(options, cmd, args);
}

fn run_with_options(options: RunOptions, cmd: String, args: Vec<String>) {
    // Set user/group if set
    match options.user.and_then(|name|users::get_user_by_name(&name)) {
        None => (),
        Some(user) => unsafe { libc::setuid(user.uid()); }
    };
    match options.group.and_then(|name|users::get_group_by_name(&name)) {
        None => (),
        Some(group) => unsafe { libc::setgid(group.gid()); }
    };
    // Setup process
    let mut proc = process::Command::new(cmd);
    proc.args(args);
    match options.env {
        None => proc.env_clear(),
        Some(e) => proc.envs(e)
    };
    proc.spawn();
}
