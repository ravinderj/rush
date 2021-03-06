extern crate dirs;

use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Error;
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::ExitStatus;

use crate::utils::get_histfile_path;

type Builtin = fn(Vec<String>) -> Result<ExitStatus, Error>;

pub fn builtins() -> HashMap<String, Builtin> {
  let mut builtins: HashMap<String, Builtin, RandomState> = HashMap::new();
  builtins.insert(String::from("cd"), builtin_cd);
  builtins.insert(String::from("exit"), builtin_exit);
  builtins.insert(String::from("let"), builtin_let);
  builtins.insert(String::from("history"), builtin_history);
  builtins
}

fn builtin_cd(args: Vec<String>) -> Result<ExitStatus, Error> {
  let path = args.get(0)
      .map(PathBuf::from)
      .unwrap_or_else(|| dirs::home_dir().unwrap());

  env::set_current_dir(path).map(|_| ExitStatus::from_raw(0))
}

fn builtin_exit(_: Vec<String>) -> Result<ExitStatus, Error> {
  std::process::exit(0)
}

fn builtin_let(args: Vec<String>) -> Result<ExitStatus, Error> {
  if args.len() != 2 {
    eprintln!("Expected 2 arguments found {}", args.len());
    Ok(ExitStatus::from_raw(1))
  } else {
    env::set_var(args.get(0).unwrap(), args.get(1).unwrap());
    Ok(ExitStatus::from_raw(0))
  }
}

fn builtin_history(_: Vec<String>) -> Result<ExitStatus, Error> {
  let history = fs::read_to_string(get_histfile_path()).unwrap_or(String::new());
  eprintln!("{}", history.trim_end());
  Ok(ExitStatus::from_raw(0))
}
