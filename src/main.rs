use std::fs::{self, File};
// use std::time::Instant;
use std::io::{BufRead, BufReader, Write, BufWriter};
use std::collections::HashSet;
use homedir::get_my_home;
use std::path::{Path, PathBuf};
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
    is_done: bool,
    id: i32,
    title: String,
    // date: Instant,
    // labels: Vec<String>,
}

//TODO check url without text tasks.

fn parse_task(line: &str) -> Option<Task> {
    if line.starts_with(TASK_DONE) || line.starts_with(TASK_UNDONE) {
        let mut line_to_parse = line;
        let status = line_to_parse.starts_with(TASK_DONE);
        //TODO: replace this by taking the length of the TASK_DONE/UNDONE.
        //otherwise, it will fail when these two are changed, though this is unlikely.
        line_to_parse = &line_to_parse[5..];
        let mut split_string = line_to_parse.split_whitespace();
        let potential_id = split_string.next()?;
        let mut id = -1;
        if potential_id.starts_with('&') {
            id = (potential_id.strip_prefix('&'))?.parse().unwrap();
        }             
        
        let task = Task {id, title:split_string.collect::<Vec<&str>>().join(" "), is_done: status};
        Some(task)
    } else {
        None
    }
}

fn task_to_string(task: &Task) -> String {
    let status_string = if task.is_done {TASK_DONE} else {TASK_UNDONE}; 
    let id_field = format!("&{}", task.id);
    format!("{} {:8} {}", status_string, id_field, task.title)
}

fn get_file_tasks(fname: &Path) -> Vec<Task> {
    let file = File::open(fname).unwrap();
    let mut file_tasks = Vec::new();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line.unwrap();
        if let Some(task) = parse_task(&line) {
            file_tasks.push(task);
        }
    }
    file_tasks
}

fn show_file_tasks(fname: &Path) {
    println!("####### {} #######", fname.to_str().expect(""));
    let file_tasks = get_file_tasks(fname);
    for task in file_tasks{
        println!("{}", task_to_string(&task));
    }
}

fn get_all_files(dir: &Path) -> Vec<PathBuf> {
    let mut all_files: Vec<PathBuf> = Vec::new();
    if dir.is_dir() {
        let mut dirs = Vec::new();
        dirs.push(dir.to_path_buf());
        while let Some(current_dir) = dirs.pop() {
            for entry in fs::read_dir(current_dir).expect("").flatten() {
                let path = entry.path().to_owned();
                if path.is_dir() {
                    dirs.push(path);
                } else {
                    all_files.push(path);
                }
            }
        }
    }
    all_files
}

struct TaskStats {
   max_id: i32, 
}

fn initialise(root_path: &Path) -> TaskStats {
    let mut stats = TaskStats{max_id:0};
    for fpath in get_all_files(root_path) {
        let ftasks = get_file_tasks(&fpath);
        for t in ftasks {
            stats.max_id = std::cmp::max(t.id, stats.max_id);
        }
    }

    // TODO: go through the tasks and set ids if not set.
    // Go through all the files and replace task lines with modified.
    // Leave non-task lines untouched.
    let mut ids: HashSet<i32> = HashSet::new();
    for fpath in get_all_files(root_path) {
        let content = fs::read_to_string(&fpath).expect("Can't read the file");
        let lines: Vec<_> = content.lines().collect();
        let of = File::create(fpath).unwrap();
        let mut writer = BufWriter::new(&of);
        for l in lines {
            if let Some(mut task) = parse_task(l) {
                if task.id < 0 && ids.contains(&task.id){
                    task.id = stats.max_id+1;
                    stats.max_id+=1;
                }
                ids.insert(task.id);
                writeln!(writer, "{}", task_to_string(&task)).unwrap();
            }
            else {
                writeln!(writer, "{}", l).unwrap();
            }
        }
         
    }
    
    stats
}

fn remove_task(task_id: i32, root_path: &Path) {
    for fpath in get_all_files(root_path) {
        let content = fs::read_to_string(&fpath).expect("Can't read the file");
        let lines: Vec<_> = content.lines().collect();
        let of = File::create(fpath).unwrap();
        let mut writer = BufWriter::new(&of);
        for l in lines {
            if let Some(task) = parse_task(l) {
                if task.id != task_id {
                    writeln!(writer, "{}", task_to_string(&task)).unwrap();
                } else {
                    println!("{}", task_to_string(&task));
                    println!("Task &{} is removed from the list", task_id);
                }
            }
            else {
                writeln!(writer, "{}", l).unwrap();
            }
        }
    }
    // todo: optimise and quit when found a task
    // print if task was not found
}

fn toggle_task(task_id: i32, root_path: &Path) {
    for fpath in get_all_files(root_path) {
        let content = fs::read_to_string(&fpath).expect("Can't read the file");
        let lines: Vec<_> = content.lines().collect();
        let of = File::create(fpath).unwrap();
        let mut writer = BufWriter::new(&of);
        for l in lines {
            if let Some(mut task) = parse_task(l) {
                if task.id == task_id {
                    task.is_done = !task.is_done;
                }
                writeln!(writer, "{}", task_to_string(&task)).unwrap();
            }
            else {
                writeln!(writer, "{}", l).unwrap();
            }
        }
    }
    // todo: optimise and quit when found a task
    // print if task was not found
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // I have no idea what's going on and why we need to unwrap twice.
    // I am also surprised that to get your home directory, you need a crate.
    let config_path = get_my_home().unwrap().unwrap();
    if config_path.exists() {
        // Initialisation starts
        // When config grows, we'll need to read file line by line.
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
        let _root_stats = initialise(root_path);
        // Initialisation ends 

        let args = Cli::parse();
        if args.command == "list" {
            println!("All gtd projects:");
            for fpath in get_all_files(root_path) {
                println!("{}", fpath.to_str().expect(""));
            }

        } else if args.command == "show" {
            let modifier_value = args.modifier.clone();
            if modifier_value.is_none() {
                show_file_tasks(&inbox_path);
            } else if modifier_value.clone().expect("") == "all" {
                for fpath in get_all_files(root_path) {
                    show_file_tasks(&fpath);
                }
            } else {
                // When we are here, we either get a folder name, or a file name.
                let mod_path = root_path.join(modifier_value.clone().expect(""));
                if modifier_value.clone().expect("").ends_with(".md") {
                    show_file_tasks(&mod_path);
                } else {
                    for fpath in get_all_files(&mod_path) {
                        show_file_tasks(&fpath);
                    }
                }
            }
        } else if args.command == "toggle" {
            let modifier_value = args.modifier.clone();
            if modifier_value.is_some() {
                let id: i32 = modifier_value.expect("Can't parse task id to toggle.").parse().unwrap();
                toggle_task(id, root_path)
            }
        } else if args.command == "rm" {
            let modifier_value = args.modifier.clone();
            if modifier_value.is_some() {
                let id: i32 = modifier_value.expect("Can't parse task id to remove.").parse().unwrap();
                remove_task(id, root_path)
            }
        }

    } else {
        println!("You need to create a config at ~/{CONFIG_FNAME} and add GTD_DIR=<rtd_root_dir_absolute_path> there.");
    }
    Ok(())
}
