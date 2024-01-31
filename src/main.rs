use std::fs::{self};
use std::io::{Write};
use homedir::get_my_home;
use std::path::Path;
use clap::Parser;

const CONFIG_FNAME: &str = ".rtd";
const INBOX_FNAME: &str = "inbox.md";
const RTD_ROOT_VAR_NAME: &str = "RTD_ROOT";

#[derive(Parser)]
struct Cli {
    command: String
}

fn visit_dirs(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Modified from the official doc: https://doc.rust-lang.org/std/fs/fn.read_dir.html.
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path)?;
            } else {
                println!("{}", path.to_str().expect(""));
            }
        }
    }
    Ok(())
}
// fn main() {
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // I have no idea what's going on and why we need to unwrap twice.
    // I am also surprised that to get your home directory, you need a crate.
    let config_path = get_my_home().unwrap().unwrap();
    if config_path.exists() {
        // Initialisation starts
        // When config will grow, we'll need to read file line by line.
        let contents = fs::read_to_string(config_path.join(CONFIG_FNAME)).expect("");
        let line: Vec<_> = contents.split('=').collect();
        if line[0] != RTD_ROOT_VAR_NAME {
            println!("You need to have {RTD_ROOT_VAR_NAME}=<absolute_path> in the config.");
        }
        let rtd_root = line[1].strip_suffix('\n').expect("");
        println!("Using rtd root: {rtd_root}");

        let root_path =  Path::new(rtd_root);
        let inbox_path =  root_path.join(INBOX_FNAME);
        if !inbox_path.exists() {
            println!("There is no {INBOX_FNAME} file in the root. Creating...");
            let mut f = fs::File::create(inbox_path)?;
            f.write_all("".as_bytes())?;
        }
        // Initialisation ends 

        let args = Cli::parse();
        if args.command == "list" {
            println!("All gtd projects:");
            let _ = visit_dirs(root_path);
        } 


    } else {
        println!("You need to create a config at ~/{CONFIG_FNAME} and add GTD_DIR=<rtd_root_dir_absolute_path> there.");
    }
    Ok(())
}
