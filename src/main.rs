use chrono::prelude::*;
use clap::Parser;
use homedir::get_my_home;
use regex::Regex;
use speedate::Date;
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

const CONFIG_FNAME: &str = ".rtd";
const INBOX_FNAME: &str = "inbox.md";
const RTD_ROOT_VAR_NAME: &str = "RTD_ROOT";
const TASK_UNDONE: &str = "- [ ]";
const TASK_DONE: &str = "- [x]";
const DONE_TASKS_FNAME: &str = ".done";
// SERVICE_FNAMES files will be ignored
// when iterating over files.
// They are used by rtd for bookkeeping.
const SERVICE_FNAMES: [&str; 1] = [DONE_TASKS_FNAME];

#[derive(Parser)]
struct Cli {
    command: String,
    modifier: Option<String>,
    submodifier: Option<String>,
}

struct TaskStats {
    max_id: i32,
}
//TODO: check if negative ids are properly processed.

struct Task {
    is_done: bool,
    id: i32,
    title: String,
    date: Option<Date>,
    labels: Vec<String>,
}

impl Task {
    fn to_string(&self) -> String {
        let mut task_string = String::from(if self.is_done { TASK_DONE } else { TASK_UNDONE });

        task_string.push_str(" &");
        task_string.push_str(&self.id.to_string());
        task_string.push(' ');
        task_string.push_str(&self.title);

        if self.date.is_some() {
            task_string.push_str(" %");
            task_string.push_str(&self.date.clone().unwrap().to_string());
        }

        if !self.labels.is_empty() {
            for l in self.labels.iter() {
                // We store labels together with the @ sign.
                task_string.push(' ');
                task_string.push_str(l);
            }
        }
        task_string
    }
}

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
        let split_string_vec = split_string.clone().collect::<Vec<&str>>();
        let mut task_body_vec: Vec<&str> = Vec::new();
        let mut task_date: Option<Date> = None;
        let mut labels: Vec<String> = Vec::new();
        if potential_id.starts_with('&') {
            id = (potential_id.strip_prefix('&'))?.parse().unwrap();
        } else {
            task_body_vec.push(potential_id);
        }
        for v in split_string_vec.iter() {
            if v.starts_with('%') {
                let date = Date::parse_str_rfc3339(v.strip_prefix('%')?);
                if date.is_ok() {
                    task_date = Some(date.expect(""));
                } else {
                    task_body_vec.push(v);
                }
            } else if v.starts_with('@') {
                labels.push(v.to_string());
            } else {
                task_body_vec.push(v);
            }
        }

        let task = Task {
            id,
            title: task_body_vec.join(" "),
            is_done: status,
            date: task_date,
            labels,
        };

        Some(task)
    } else {
        None
    }
}

fn today() -> Date {
    let today = Local::now().format("%Y-%m-%d");
    Date::parse_str_rfc3339(&today.to_string()).expect("Can't parse today's date.")
}

fn get_file_tasks(fname: &Path, due_only: bool, label: Option<String>) -> Vec<Task> {
    let file = File::open(fname).unwrap();
    let mut file_tasks = Vec::new();
    let reader = BufReader::new(file);
    let speedate_today = today();
    for line in reader.lines() {
        let line = line.unwrap();
        if let Some(task) = parse_task(&line) {
            if due_only && (task.date.is_none() || task.date.clone().unwrap() > speedate_today) {
                continue;
            }
            if label.is_some() && !task.labels.contains(&label.clone().unwrap()) {
                continue;
            }
            file_tasks.push(task);
        }
    }
    file_tasks
}

fn show_file_tasks(fname: &Path, due_only: bool, label: Option<String>) {
    let file_tasks = get_file_tasks(fname, due_only, label);
    if !file_tasks.is_empty() {
        println!("####### {} #######", fname.to_str().expect(""));
    }
    for task in file_tasks {
        println!("{}", task.to_string());
    }
}

fn get_file_labels(fname: &Path) -> HashSet<String> {
    let mut labels: HashSet<String> = HashSet::new();
    let file_tasks = get_file_tasks(fname, false, None);
    for t in file_tasks {
        for l in t.labels {
            labels.insert(l);
        }
    }
    labels
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
                    let mut is_service = false;
                    for el in SERVICE_FNAMES {
                        // Ideally, this el has to be joined with root_path,
                        // but I was lazy.
                        if path.ends_with(el) {
                            is_service = true;
                            break;
                        }
                    }
                    if !is_service {
                        all_files.push(path);
                    }
                }
            }
        }
    }
    all_files
}

fn initialise(root_path: &Path) -> TaskStats {
    let mut stats = TaskStats { max_id: 0 };
    for fpath in get_all_files(root_path) {
        let ftasks = get_file_tasks(&fpath, false, None);
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
                if task.id < 0 && ids.contains(&task.id) {
                    task.id = stats.max_id + 1;
                    stats.max_id += 1;
                }
                ids.insert(task.id);
                writeln!(writer, "{}", task.to_string()).unwrap();
            } else {
                writeln!(writer, "{}", l).unwrap();
            }
        }
    }

    let done_file_path = root_path.join(DONE_TASKS_FNAME);
    if !done_file_path.exists() {
        println!("There is no {DONE_TASKS_FNAME} file in the root. Creating...");
        let mut f = fs::File::create(done_file_path.clone()).unwrap();
        f.write_all("".as_bytes()).expect("");
    }

    stats
}

fn move_task(task_id: i32, root_path: &Path, dest_fpath: &Path) {
    let mut found = false;
    let dest_path = root_path.join(dest_fpath);
    if !dest_path.exists() {
        eprintln!("Destination file does not exist.");
        return;
    }
    for fpath in get_all_files(root_path) {
        let content = fs::read_to_string(&fpath).expect("Can't read the file");
        let lines: Vec<_> = content.lines().collect();
        let of = File::create(fpath).unwrap();
        let mut writer = BufWriter::new(&of);
        for l in lines {
            if let Some(task) = parse_task(l) {
                if task.id != task_id {
                    writeln!(writer, "{}", l).unwrap();
                } else {
                    println!("{}", task.to_string());
                    println!(
                        "Task &{} is moved to the list {}",
                        task_id,
                        dest_fpath.to_str().unwrap()
                    );
                    let mut dest_file = OpenOptions::new()
                        .append(true)
                        .open(root_path.join(dest_fpath))
                        .unwrap();
                    writeln!(dest_file, "{l}").unwrap();
                    found = true;
                }
            } else {
                writeln!(writer, "{}", l).unwrap();
            }
        }
        if found {
            break;
        }
        //Do I need to close the files in rust?
    }
    if !found {
        println!("Task &{} is not in any of your files", task_id);
    };
}

fn remove_task(task_id: i32, root_path: &Path) {
    let mut found = false;
    for fpath in get_all_files(root_path) {
        let content = fs::read_to_string(&fpath).expect("Can't read the file");
        let lines: Vec<_> = content.lines().collect();
        let of = File::create(fpath).unwrap();
        let mut writer = BufWriter::new(&of);
        for l in lines {
            if let Some(task) = parse_task(l) {
                if task.id != task_id {
                    writeln!(writer, "{}", task.to_string()).unwrap();
                } else {
                    println!("{}", task.to_string());
                    println!("Task &{} is removed from the list", task_id);
                    found = true;
                }
            } else {
                writeln!(writer, "{}", l).unwrap();
            }
        }
        if found {
            break;
        }
    }
    if !found {
        println!("Task &{} is not in any of your files", task_id);
    };
}

fn add_task(task_str: &str, fpath: &Path, mut stats: TaskStats) {
    let content = fs::read_to_string(fpath).expect("Can't read the file");
    let lines: Vec<_> = content.lines().collect();
    let of = File::create(fpath).unwrap();
    let mut writer = BufWriter::new(&of);
    //todo append status string here
    let mut task_string = String::from(TASK_UNDONE);
    task_string.push(' ');
    task_string.push_str(task_str);
    let mut task_to_write = parse_task(&task_string).unwrap();
    task_to_write.id = stats.max_id + 1;
    stats.max_id += 1;
    println!("Added new task to {}:", fpath.to_str().unwrap());
    println!("{}", task_to_write.to_string());
    writeln!(writer, "{}", task_to_write.to_string()).unwrap();
    for l in lines {
        writeln!(writer, "{}", l).unwrap();
    }
}

fn toggle_task(task_id: i32, root_path: &Path, toggle_status: bool, toggle_date: bool) {
    for fpath in get_all_files(root_path) {
        let content = fs::read_to_string(&fpath).expect("Can't read the file");
        let lines: Vec<_> = content.lines().collect();
        let of = File::create(fpath).unwrap();
        let mut writer = BufWriter::new(&of);
        for l in lines {
            if let Some(mut task) = parse_task(l) {
                if task.id == task_id {
                    if toggle_status {
                        task.is_done = !task.is_done;
                        println!("Changed status of the task {}", task_id);
                        println!("Current state:");
                        println!("{}", task.to_string());
                    }
                    if toggle_date {
                        if task.date.is_some() {
                            task.date = None;
                        } else {
                            task.date = Some(today());
                        }
                    }
                }
                writeln!(writer, "{}", task.to_string()).unwrap();
            } else {
                writeln!(writer, "{}", l).unwrap();
            }
        }
    }
    // todo: optimise and quit when found a task
    // print if task was not found
}

// TODO: make a general 'modify_task' function that takes
// options for each of the task fields.
// This will allow to get rid of all the similar functions
// that iterate over files and find one id.
// Another thing to do will be to keep a hashmap of the file/ids
// when initialising, and use this to find a file to write to.
// These two are complementary to each other.
fn add_label_to_task(task_id: i32, root_path: &Path, label: String) {
    for fpath in get_all_files(root_path) {
        let content = fs::read_to_string(&fpath).expect("Can't read the file");
        let lines: Vec<_> = content.lines().collect();
        let of = File::create(fpath).unwrap();
        let mut writer = BufWriter::new(&of);
        for l in lines {
            if let Some(mut task) = parse_task(l) {
                if task.id == task_id {
                    task.labels.push(label.clone());
                }
                writeln!(writer, "{}", task.to_string()).unwrap();
            } else {
                writeln!(writer, "{}", l).unwrap();
            }
        }
    }
    // todo: optimise and quit when found a task
    // print if task was not found
}

fn get_task(task_id: i32, root_path: &Path) -> Option<Task> {
    for fpath in get_all_files(root_path) {
        let content = fs::read_to_string(&fpath).expect("Can't read the file");
        let lines: Vec<_> = content.lines().collect();
        for l in lines {
            if let Some(task) = parse_task(l) {
                if task.id == task_id {
                    return Some(task);
                }
            }
        }
    }
    None
}

fn archive_tasks(root_path: &Path) {
    let mut done_file = OpenOptions::new()
        .append(true)
        .open(root_path.join(DONE_TASKS_FNAME))
        .unwrap();

    for fpath in get_all_files(root_path) {
        let content = fs::read_to_string(&fpath).expect("Can't read the file");
        let lines: Vec<_> = content.lines().collect();
        let of = File::create(fpath.clone()).unwrap();
        let mut writer = BufWriter::new(&of);
        for l in lines {
            if let Some(task) = parse_task(l) {
                let mut task_string = task.to_string();
                if task.is_done {
                    task_string.push(' ');
                    task_string.push_str(fpath.to_str().unwrap());
                    writeln!(done_file, "{task_string}").unwrap();
                } else {
                    writeln!(writer, "{task_string}").unwrap();
                }
            } else {
                writeln!(writer, "{}", l).unwrap();
            }
        }
    }
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

        let root_path = Path::new(rtd_root);
        let inbox_path = root_path.join(INBOX_FNAME);
        if !inbox_path.exists() {
            println!("There is no {INBOX_FNAME} file in the root. Creating...");
            let mut f = fs::File::create(inbox_path.clone())?;
            f.write_all("".as_bytes())?;
        }
        let root_stats = initialise(root_path);
        // Initialisation ends

        let args = Cli::parse();
        let maybe_path = root_path.join(args.command.clone());

        if args.command == "debug" {
            println!("Using rtd root: {rtd_root}.");
        } else if args.command.parse::<i32>().is_ok() {
            let id = args.command.parse::<i32>().unwrap();
            let task = get_task(id, root_path);
            if task.is_some() {
                println!("{}", &task.unwrap().to_string())
            }
        } else if args.command == "url" {
            let modifier_value = args.modifier.clone();
            if modifier_value.is_some() {
                let id = modifier_value
                    .unwrap()
                    .parse::<i32>()
                    .expect("Provide task id for the url command.");
                let task = get_task(id, root_path);
                if task.is_some() {
                    let re = Regex::new(r"http://\S+|https://\S+").unwrap();
                    for cap in re.captures_iter(&(&task.unwrap().to_string())) {
                        println!("{}", &cap[0]);
                    }
                }
            }
        } else if args.command == "list" {
            let root_path_str = root_path.to_str().unwrap().to_string();
            println!("All gtd projects:");
            for fpath in get_all_files(root_path) {
                println!(
                    "{}",
                    fpath
                        .to_str()
                        .expect("")
                        .strip_prefix(&root_path_str)
                        .unwrap()
                );
            }

        //TODO: Check files for keywords and throw an error
        // if there are folders with names due/labels etc.
        } else if args.command == "labels" {
            let mut all_labels: HashSet<String> = HashSet::new();
            for fpath in get_all_files(root_path) {
                all_labels.extend(get_file_labels(&fpath));
            }
            for l in all_labels {
                println!("{l}");
            }
        } else if args.command.starts_with('@') {
            for fpath in get_all_files(root_path) {
                show_file_tasks(&fpath, false, Some(args.command.clone()));
            }
        } else if maybe_path.exists() {
            // When we are here, we either get a folder name, or a file name.
            if maybe_path.clone().to_str().unwrap().ends_with(".md") {
                show_file_tasks(&maybe_path, false, None);
            } else {
                for fpath in get_all_files(&maybe_path) {
                    show_file_tasks(&fpath, false, None);
                }
            }
        } else if args.command == "inbox" || args.command == "i" {
            show_file_tasks(&inbox_path, false, None);
        } else if args.command == "due" {
            for fpath in get_all_files(root_path) {
                show_file_tasks(&fpath, true, None);
            }
        } else if args.command == "all" {
            for fpath in get_all_files(root_path) {
                show_file_tasks(&fpath, false, None);
            }
        } else if args.command == "toggle" || args.command == "td" {
            let modifier_value = args.modifier.clone();
            if modifier_value.is_some() {
                let id: i32 = modifier_value
                    .expect("Can't parse task id.")
                    .parse()
                    .unwrap();
                if args.command == "toggle" {
                    toggle_task(id, root_path, true, false);
                } else if args.command == "td" {
                    toggle_task(id, root_path, false, true);
                }
            }
        } else if args.command == "al" {
            let modifier_value = args.modifier.clone();
            if modifier_value.is_some() {
                let id: i32 = modifier_value
                    .expect("Can't parse task id.")
                    .parse()
                    .unwrap();
                let submodifier_value = args.submodifier.clone();
                if submodifier_value.is_some() {
                    let label_str = submodifier_value.unwrap().to_string();
                    if label_str.starts_with('@') {
                        add_label_to_task(id, root_path, label_str);
                    } else {
                        eprintln!("A label should start with @ and have no spaces in it.");
                    }
                } else {
                    eprintln!("Provide a label to add.");
                }
            } else {
                eprintln!("Provide task id to add the label to.");
            }
        } else if args.command == "rm" {
            let modifier_value = args.modifier.clone();
            if modifier_value.is_some() {
                let id: i32 = modifier_value
                    .expect("Can't parse task id to remove.")
                    .parse()
                    .unwrap();
                remove_task(id, root_path);
            }
        } else if args.command == "add" {
            let modifier_value = args.modifier.clone();
            if modifier_value.is_some() {
                let submodifier_value = args.submodifier.clone();
                if submodifier_value.is_some() {
                    let fpath = root_path.join(submodifier_value.unwrap());
                    add_task(&modifier_value.unwrap(), &fpath, root_stats);
                } else {
                    add_task(&modifier_value.unwrap(), &inbox_path, root_stats);
                }
            } else {
                println!("Specify a task to add!");
            }
        } else if args.command == "archive" {
            archive_tasks(root_path);
            println!("All tasks archived (moved to .done)");
        } else if args.command == "mv" {
            let modifier_value = args.modifier.clone();
            if modifier_value.is_some() {
                let id: i32 = modifier_value
                    .expect("Can't parse task id to move.")
                    .parse()
                    .unwrap();
                let submodifier_value = args
                    .submodifier
                    .expect("Please provide a destination file to move the task to.");
                let dest_fpath = Path::new(&submodifier_value);
                move_task(id, root_path, dest_fpath);
            }
        } else {
            println!("Unknown command: {}", args.command);
        }
    } else {
        println!("You need to create a config at ~/{CONFIG_FNAME} and add GTD_DIR=<rtd_root_dir_absolute_path> there.");
    }
    Ok(())
}
