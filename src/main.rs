extern crate chrono;

use std::env;
use std::fs::OpenOptions;
use std::io::{Error, Write};
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, ExitStatus, Stdio, Child};

use rush::builtins::builtins;
use rush::input::*;
use rush::output::println_err;
use rush::rush::Rush;
use rush::utils::get_histfile_path;

fn main() {
  let greeting = "Welcome to RUSH.......";
  println!("{}", greeting);

  loop {
    let input = read_line(build_prompt());
    match launch(Rush::from(input.clone())) {
      Ok(status) => {
        if let Some(code) = status.code() {
          env::set_var("STATUS", code.to_string())
        }
      }
      Err(_) => {
        env::set_var("STATUS", 127.to_string());
        println_err("Command not found".to_owned())
      }
    }
    save_history(input);
  }
}

fn build_prompt() -> String {
  let prompt = "≈>";
  env::current_dir().unwrap().to_string_lossy().to_string() + prompt
}

fn launch(command: Rush) -> Result<ExitStatus, Error> {
  match command {
    Rush::Bin(cmd, args) => {
      builtins()
          .get(&cmd)
          .map_or_else(|| execute(cmd, args.clone()),
                       |builtin| builtin(args.clone()))
    }
    Rush::Empty => Ok(ExitStatus::from_raw(0)),
    Rush::Piped(mut commands) => {
      let last = commands.pop();
      let x = commands
          .iter()
          .fold(None, |r: Option<Child>, c| {
            let stdin = r.map(|c| Stdio::from(c.stdout.unwrap()))
                .unwrap_or(Stdio::inherit());
            spawn_c(c, stdin, Stdio::piped())
          })
          .unwrap();

      spawn_c(&last.unwrap(), Stdio::from(x.stdout.unwrap()), Stdio::inherit())
          .unwrap()
          .wait()
    }
  }
}

fn spawn_c(r: &Rush, stdin: Stdio, stdout: Stdio) -> Option<Child> {
  match r {
    Rush::Bin(cmd, args) => spawn(cmd, args, stdin, stdout).ok(),
    _ => None
  }
}

fn execute(cmd: String, args: Vec<String>) -> Result<ExitStatus, Error> {
  spawn(&cmd, &args, Stdio::inherit(), Stdio::inherit())
      .map(|mut c| c.wait())?
}

fn spawn(cmd: &String, args: &Vec<String>, stdin: Stdio, stdout: Stdio) -> Result<Child, Error> {
  Command::new(cmd)
      .args(args)
      .stdin(stdin)
      .stdout(stdout)
      .spawn()
}

fn save_history(cmd: String) {
  let timestamp = chrono::Local::now().format("%s").to_string();

  match cmd.as_str() {
    "" => (),
    _ => {
      let mut file = OpenOptions::new()
        .create(true).append(true)
        .open(get_histfile_path()).unwrap();
      file.write((timestamp + ";" + cmd.as_str() + "\n").as_bytes()).unwrap();
    }
  }
}
