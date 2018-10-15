extern crate users;
extern crate nix;

#[macro_use]
extern crate chan;
extern crate chan_signal;

use std::process;
use std::thread;
use std::fmt;
use std::collections::HashMap;
use chan_signal::Signal;

struct RunOptions<'a> {
    env: Option<&'a HashMap<&'a str, &'a str>>,
    user: Option<&'a str>,
    group: Option<&'a str>,
    work_dir: Option<&'a str>,
    exit_timeout: i64
}

#[derive(Debug)]
enum RunError {
    Nix(nix::Error),
    Io(std::io::Error),
    Process(std::io::Error)// ,
    // CouldntSetUid(uid),
    // CouldntSetGid(gid)
}

impl std::convert::From<std::io::Error> for RunError {
    fn from(e: std::io::Error) -> RunError {
        RunError::Io(e)
    }
}

impl std::convert::From<nix::Error> for RunError {
    fn from(e: nix::Error) -> RunError {
        RunError::Nix(e)
    }
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RunError::Nix(err) => write!(f, "UNIX call: {}", err),
            RunError::Io(err) => write!(f, "I/O call: {}", err),
            RunError::Process(err) => write!(f, "Process error: {}", err)
        }
    }
}

#[derive(Debug)]
enum RunResult {
    Exited(process::ExitStatus),
    Signalled(Option<chan_signal::Signal>),
    ChanEnded
}

fn main() {
    let args = vec![
        String::from("10")
    ];
    println!("Sleeping for 10...");
    match run(String::from("sleep"), args, None) {
        Ok(run_result) =>
            match run_result {
                RunResult::Exited(status) => println!("Process exited with status: {}", status),
                RunResult::Signalled(signal) =>
                    match signal {
                        None => println!("Process was signalled to end (signal unknown)."),
                        Some(sig) => println!("Process was signalled to end with signal: {:?}", sig)
                    }
                RunResult::ChanEnded => println!("Thread running the process stopped sending updates!")
            },
        Err(err) => println!("Error in process: {}", err)
    }
}

fn run(cmd: String,
       args: Vec<String>,
       env: Option<&HashMap<&str, &str>>) -> Result<RunResult, RunError> {
    let options: RunOptions = RunOptions {
        env: env,
        user: Some("hello"),
        group: None,
        work_dir: None,
        exit_timeout: 5
    };
    run_with_options(options, cmd, args)
}

fn run_with_options(options: RunOptions, cmd: String, args: Vec<String>) -> Result<RunResult, RunError> {
    // Set user/group if set
    // ANNOYING: (what's going on with this closure and returning?)
    // FIXME: don't use and_then here:
    // - need to pattern match on the None case (user-provided)
    // - return from inside a closure just exits the closure
    match options.user.and_then(|name|users::get_user_by_name(name)) {
        None => (),
        Some(user) => nix::unistd::setuid(nix::unistd::Uid::from_raw(user.uid()))?
    };
    match options.group.and_then(|name|users::get_group_by_name(name)) {
        None => (),
        Some(group) => nix::unistd::setgid(nix::unistd::Gid::from_raw(group.gid()))?
    };
    match options.work_dir {
        None => (),
        Some(wdir) => {
            std::env::set_current_dir(&(std::path::Path::new(wdir)))?
        }
    }
    if process::id() == 1 {
        println!("Running as pid1 ...");
        run_as_pid1(cmd, args, options.env, options.exit_timeout)
    } else {
        execute_file(cmd, args, options.env).map(RunResult::Exited)
    }
}

// FIXME: https://docs.rs/nix/0.10.0/nix/unistd/fn.execve.html
fn execute_file(cmd: String, args: Vec<String>, env: Option<&HashMap<&str, &str>>) -> Result<process::ExitStatus, RunError> {
    let mut pro = process::Command::new(cmd);
    pro.args(args);
    match env {
        None => pro.env_clear(),
        Some(e) => pro.envs(e)
    };
    pro.status().map_err(RunError::Process)
}

fn run_as_pid1(cmd: String, args: Vec<String>, env: Option<&HashMap<&str,&str>>, timeout: i64) -> Result<RunResult, RunError> {
    // Signal gets a value when the OS sent a INT or TERM signal.
    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);
    // When our work is complete, send a sentinel value on `sdone`.
    let (sdone, rdone) = chan::sync(0);

    let env2 = env.map(|x|x.iter().map(|(k,v)|(k.to_string(),v.to_string())).collect());
    // Run work.
    thread::spawn(move || match env2 {
        Some(env) => execute_with_sender(sdone, cmd, args, Some(&env), timeout),
        None => execute_with_sender(sdone, cmd, args, None, timeout)
    });

    // Wait for a signal or for work to be done.
    chan_select! {
        signal.recv() -> signal => {
            return Ok(RunResult::Signalled(signal));
        },
        rdone.recv() -> result => {
            match result {
                None => return Ok(RunResult::ChanEnded),
                Some(exit_result) => {
                    return exit_result.map(RunResult::Exited).map_err(RunError::Process)
                }
            }
        }
    }
}

fn execute_with_sender(sender: chan::Sender<std::result::Result<std::process::ExitStatus, std::io::Error>>, cmd: String, args: Vec<String>, env: Option<&HashMap<&str, &str>>, _timeout: i64) {
    let mut pro = process::Command::new(cmd);
    pro.args(args);
    match env {
        None => pro.env_clear(),
        Some(e) => pro.envs(e)
    };
    sender.send(pro.status())
}
