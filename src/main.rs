use std::fs;
use homedir::get_my_home;

const CONFIG_FNAME: &str = ".rtd";
const RTD_ROOT_VAR_NAME: &str = "RTD_ROOT";

fn main() {
    // I have no idea what's going on and why we need to unwrap twice.
    // I am also surprised that to get your home directory, you need a crate.
    let config_path = get_my_home().unwrap().unwrap().join(CONFIG_FNAME);
    if config_path.exists() {
        // When config will grow, we'll need to read file line by line.
        let contents = fs::read_to_string(config_path.as_path()).expect("");
        // TODO: add index.md if it does not exist.
        println!("{contents}");
        let line: Vec<_> = contents.split("=").collect();
        if line[0] != RTD_ROOT_VAR_NAME {
            println!("You need to have {RTD_ROOT_VAR_NAME}=<absolute_path> in the config.");
        }
        let rtd_root = line[1];
        println!("Using rtd root: {rtd_root}");
    } else {
        println!("You need to create a config at ~/{CONFIG_FNAME} and add GTD_DIR=<rtd_root_dir_absolute_path> there.");
    }
}
