use std::fs::{self};
use std::io::{Write};
// use std::time::Instant;
use std::io::{BufRead, BufReader};
use homedir::get_my_home;
use std::path::Path;
use clap::Parser;

const CONFIG_FNAME: &str = ".rtd";
const INBOX_FNAME: &str = "inbox.md";
const RTD_ROOT_VAR_NAME: &str = "RTD_ROOT";
const TASK_UNDONE: &str = "- [ ]";
const TASK_DONE: &str = "- [x]";

#[derive(Parser)]
struct Cli {
    command: String,
    modifier: Option<String>,
}

struct Task {
    id: i32,
    title: String,
    // date: Instant,
    // labels: Vec<String>,
}

fn parse_task(line: &str) -> Option<Task> {
    // check that line starts with TASK_UNDONE or TASK_DONE
    // if yes, put the rest into title for now
    if line.starts_with(TASK_DONE) || line.starts_with(TASK_UNDONE) {
        let task = Task {id: 0, title:line.to_string()};
        Some(task)
    } else {
        None
    }
}

fn get_all_files(dir: &Path) -> Vec<String> {
    let mut all_files: Vec<String> = Vec::new();
    if dir.is_dir() {
        let mut dirs = Vec::new();
        dirs.push(dir.to_path_buf());
        while let Some(current_dir) = dirs.pop() {
            for entry in fs::read_dir(current_dir).expect("").flatten() {
                let path = entry.path().to_owned();
                if path.is_dir() {
                    dirs.push(path);
                } else {
                    all_files.push(path.to_str().expect("").to_string());
                }
            }
        }
    }
    all_files
}

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
            let mut f = fs::File::create(inbox_path.clone())?;
            f.write_all("".as_bytes())?;
        }
        // Initialisation ends 

        let args = Cli::parse();
        if args.command == "list" {
            println!("All gtd projects:");
            for fname in get_all_files(root_path) {
                println!("{fname}");
            }

        } else if args.command == "show" {
            if args.modifier.is_none() {
                let file = fs::File::open(inbox_path).unwrap();
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    let line = line.unwrap();
                    // Show the line and its number.
                    let task = parse_task(&line).unwrap();
                    println!("{}: {}", task.id, task.title);
                }
            } else if args.modifier.expect("") == "all" {
                    println!("Showing all todos.")
            }
        }


    } else {
        println!("You need to create a config at ~/{CONFIG_FNAME} and add GTD_DIR=<rtd_root_dir_absolute_path> there.");
    }
    Ok(())
}
