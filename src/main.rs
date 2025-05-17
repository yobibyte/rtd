use chrono::prelude::*;
use clap::{Parser, Subcommand};
use regex::Regex;
use speedate::Date;
use std::collections::HashSet;
use std::fmt;
use std::env;
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
// SERVICE_FNAMES files will be ignored when iterating over files.
// They are used by rtd for bookkeeping.
const SERVICE_FNAMES: [&str; 1] = [DONE_TASKS_FNAME];

#[derive(Parser)]
#[command(subcommand_required = false, arg_required_else_help = false)]
struct Cli {
    #[command(subcommand)]
    command: Option<SubcommandEnum>,
    /// This can be a task id, @label or a project (e.g. file.md).
    global_modifier: Option<String>,
}

#[derive(Debug, Subcommand)]
enum SubcommandEnum {
    ///Show inbox.
    #[command(visible_alias = "i")]
    Inbox,
    /// Show all tasks in the workspace.
    #[command(visible_alias = "a")]
    All,
    /// Show due tasks.
    Due,
    ///Print an URL if a task description has one. Provide task id.
    Url { task_id: i32 },
    ///Remove task. Provide task id.
    Rm { task_id: i32 },
    ///Print out a list of all projects.
    List,
    ///Show all labels.
    Labels,
    ///Add a task. <task_description> <project>. If project not provided, adding to inbox. Task
    ///description can have a date (starts with %), and labels (each starts with @, no spaces
    ///allowed).
    Add {
        task_description: String,
        project: Option<String>,
    },
    ///Add a label to a task. <task_id> <label>. Label starts with @.
    #[command(visible_alias = "al")]
    AddLabel { task_id: i32, label: String },
    ///Move a task to a project: <task_id> <project>.
    Mv { task_id: i32, project: String },
    ///Move done tasks to archive.
    Archive,
    ///Toggle task status (done -> undone, undone -> done).
    #[command(visible_alias = "t")]
    Toggle { task_id: i32 },
    ///Toggle task date (change for today!)
    #[command(visible_alias = "td")]
    ToggleDate { task_id: i32 },
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

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.is_done { TASK_DONE } else { TASK_UNDONE };
        write!(f, "{} &{} {}", status, self.id, self.title)?;

        if let Some(date) = &self.date {
            write!(f, " %{}", date.clone())?;
        }

        if !self.labels.is_empty() {
            for l in self.labels.iter() {
                // We store labels together with the @ sign.
                write!(f, " {}", l).expect("Failed to write to task_string");
            }
        }
        Ok(())
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
        println!("{}", task);
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
                writeln!(writer, "{}", task).unwrap();
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
                    println!("{}", task);
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
                    writeln!(writer, "{}", task).unwrap();
                } else {
                    println!("{}", task);
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
    println!("{}", task_to_write);
    writeln!(writer, "{}", task_to_write).unwrap();
    for l in lines {
        writeln!(writer, "{}", l).unwrap();
    }
}

fn modify_task(
    task_id: i32,
    root_path: &Path,
    label_to_add: Option<String>,
    toggle_status: bool,
    toggle_date: bool,
) {
    for fpath in get_all_files(root_path) {
        let content = fs::read_to_string(&fpath).expect("Can't read the file");
        let lines: Vec<_> = content.lines().collect();
        let of = File::create(fpath).unwrap();
        let mut writer = BufWriter::new(&of);
        for l in lines {
            if let Some(mut task) = parse_task(l) {
                if task.id == task_id {
                    // This branch is doing all the modifications.
                    // If the argument is Some, update the task with it.
                    if let Some(label) = label_to_add.clone() {
                        task.labels.push(label);
                    }
                    if toggle_status {
                        task.is_done = !task.is_done;
                        println!("Changed status of the task {}", task_id);
                        println!("Current state:");
                        println!("{}", task);
                    }
                    if toggle_date {
                        if task.date.is_some() {
                            task.date = None;
                        } else {
                            task.date = Some(today());
                        }
                    }
                }
                writeln!(writer, "{}", task).unwrap();
            } else {
                writeln!(writer, "{}", l).unwrap();
            }
        }
    }
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
    let config_path = env::home_dir().expect("I need a $HOME to operate.");
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
        match args.command {
            Some(subcommand) => match subcommand {
                SubcommandEnum::All => {
                    for fpath in get_all_files(root_path) {
                        show_file_tasks(&fpath, false, None);
                    }
                }
                SubcommandEnum::Inbox => {
                    show_file_tasks(&inbox_path, false, None);
                }
                SubcommandEnum::Due => {
                    for fpath in get_all_files(root_path) {
                        show_file_tasks(&fpath, true, None);
                    }
                }
                SubcommandEnum::Archive => {
                    archive_tasks(root_path);
                    println!("All tasks archived (moved to .done)");
                }
                SubcommandEnum::Labels => {
                    let mut all_labels: HashSet<String> = HashSet::new();
                    for fpath in get_all_files(root_path) {
                        all_labels.extend(get_file_labels(&fpath));
                    }
                    for l in all_labels {
                        println!("{l}");
                    }
                }
                SubcommandEnum::List => {
                    let root_path_str = root_path.to_str().unwrap().to_string();
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
                }
                SubcommandEnum::Url { task_id } => {
                    let task = get_task(task_id, root_path);
                    if task.is_some() {
                        let re = Regex::new(r"http://\S+|https://\S+").unwrap();
                        for cap in re.captures_iter(&task.unwrap().to_string()) {
                            println!("{}", &cap[0]);
                        }
                    }
                }
                SubcommandEnum::Rm { task_id } => remove_task(task_id, root_path),
                SubcommandEnum::Toggle { task_id } => {
                    modify_task(task_id, root_path, None, true, false)
                }
                SubcommandEnum::ToggleDate { task_id } => {
                    modify_task(task_id, root_path, None, false, true)
                }
                SubcommandEnum::Add {
                    task_description,
                    project,
                } => {
                    let project_path = match project {
                        Some(project) => root_path.join(&project),
                        None => root_path.join(&inbox_path),
                    };
                    add_task(&task_description, &project_path, root_stats);
                }
                SubcommandEnum::Mv { task_id, project } => {
                    move_task(task_id, root_path, Path::new(&project))
                }
                SubcommandEnum::AddLabel { task_id, label } => {
                    if label.starts_with('@') {
                        modify_task(task_id, root_path, Some(label), false, false);
                    } else {
                        eprintln!("A label should start with @ and have no spaces in it.");
                    }
                }
            },
            None => match args.global_modifier {
                Some(modifier) => {
                    let maybe_path = root_path.join(modifier.clone());
                    if let Ok(id) = modifier.parse::<i32>() {
                        let task = get_task(id, root_path);
                        if task.is_some() {
                            println!("{}", &task.unwrap().to_string())
                        }
                    } else if modifier.starts_with('@') {
                        for fpath in get_all_files(root_path) {
                            show_file_tasks(&fpath, false, Some(modifier.clone()));
                        }
                    //TODO: Check files for keywords and throw an error
                    // if there are folders with names due/labels etc.
                    } else if maybe_path.exists() {
                        // When we are here, we either get a folder name, or a file name.
                        if maybe_path.clone().to_str().unwrap().ends_with(".md") {
                            show_file_tasks(&maybe_path, false, None);
                        } else {
                            for fpath in get_all_files(&maybe_path) {
                                show_file_tasks(&fpath, false, None);
                            }
                        }
                    } else {
                        println!("Unknown modifier: {}", modifier);
                    }
                }
                None => {
                    for fpath in get_all_files(root_path) {
                        show_file_tasks(&fpath, false, None);
                    }
                }
            },
        }
    } else {
        println!("You need to create a config at ~/{CONFIG_FNAME} and add GTD_DIR=<rtd_root_dir_absolute_path> there.");
    }
    Ok(())
}
