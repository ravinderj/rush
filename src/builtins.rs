extern crate dirs;

use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::env;
use std::io::Error;
use std::path::PathBuf;
use std::process::{Output, ExitStatus};
use std::os::unix::process::ExitStatusExt;

type Builtin = fn(Vec<String>) -> Result<Output, Error>;

pub fn builtins() -> HashMap<String, Builtin> {
  let mut builtins: HashMap<String, Builtin, RandomState> = HashMap::new();
  builtins.insert(String::from("cd"), builtin_cd);
  builtins.insert(String::from("exit"), builtin_exit);
  builtins.insert(String::from("let"), builtin_let);
  builtins
}

fn builtin_cd(args: Vec<String>) -> Result<Output, Error> {
  let path = args.get(0)
      .map(PathBuf::from)
      .unwrap_or_else(|| dirs::home_dir().unwrap());

  env::set_current_dir(path).map(|_| Output {
    stderr: Vec::new(),
    stdout: Vec::new(),
    status: ExitStatus::from_raw(0),
  })
}

fn builtin_exit(_: Vec<String>) -> Result<Output, Error> {
  std::process::exit(0)
}

fn builtin_let(args: Vec<String>) -> Result<Output, Error> {
  if let Some(binding) = args.get(0) {
    let mut key_val = binding.split('=');
    env::set_var(key_val.next().unwrap(), key_val.next().unwrap())
  }

  Ok(Output {
    stderr: Vec::new(),
    stdout: Vec::new(),
    status: ExitStatus::from_raw(0),
  })
}
