mod lib;
use std::env;
use std::fs;

use crate::lib::{get_compose_file, get_compose_map};

fn main() -> Result<(), serde_yaml::Error> {
    let args: Vec<String> = env::args().collect();
    let filename = get_compose_file(&args);
    let yaml = fs::read_to_string(filename)
        .expect("Should have been able to read the file");
    let map = get_compose_map(&yaml)?;
    let services = &map["services"].as_mapping().unwrap();
    println!("{:?}", services.keys().collect::<Vec<_>>());
    Ok(())
}
