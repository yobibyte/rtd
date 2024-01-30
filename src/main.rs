use std::fs;
use homedir::get_my_home;

fn main() {
    // I have no idea what's going on and why we need to unwrap twice.
    // I am also surprised that to get your home directory, you need a crate.
    let config_path = get_my_home().unwrap().unwrap().join(".rtd");
    // TODO: create a file if it does not exist.
    let contents = fs::read_to_string(config_path.as_path()).expect("");
    // TODO: add index.md if it does not exist.
    println!("{contents}")
}
