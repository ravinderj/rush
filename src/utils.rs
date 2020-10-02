use std::env;

pub fn get_histfile_path() -> String {
    let default_path = format!("{}/.rush_history", env::var("HOME").unwrap());
    return env::var("HISTFILE").unwrap_or(default_path);
}