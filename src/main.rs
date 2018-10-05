extern crate users;
extern crate libc;

use std::process;
use std::env;

struct RunOptions {
    env: Option<Vec<(String, String)>>,
    user: Option<String>,
    group: Option<String>,
    work_dir: Option<String>,
    exit_timeout: i64
}

fn main() {
    let args = vec![String::from("-f"),String::from("x.txt")];
    run(String::from("tail"), args, None);
}

fn run(cmd: String,
       args: Vec<String>,
       env: Option<Vec<(String, String)>>) {
    let options: RunOptions = RunOptions {
        env: env,
        user: None,
        group: None,
        work_dir: None,
        exit_timeout: 5
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
    match options.work_dir {
        None => (),
        Some(wdir) => {
            let root = std::path::Path::new(&wdir);
            std::env::set_current_dir(&root);
        }
    }
    if process::id() == 1 {
        run_as_pid1(cmd, args, options.env, options.exit_timeout);
    } else {
        execute_file(cmd, args, options.env);
    }
}

fn execute_file(cmd: String, args: Vec<String>, env: Option<Vec<(String, String)>>) {
    let mut proc = process::Command::new(cmd);
    proc.args(args);
    match env {
        None => proc.env_clear(),
        Some(e) => proc.envs(e)
    };
    match proc.status().expect("Failed to execute").code() {
        Some(code) => process::exit(code),
        None       => ()
    }
}

fn run_as_pid1(cmd: String, args: Vec<String>, env: Option<Vec<(String, String)>>, timeout: i64) {
    let mut proc = process::Command::new(cmd);
    proc.args(args);
    match env {
        None => proc.env_clear(),
        Some(e) => proc.envs(e)
    };
    proc.spawn();
}
