mod lib;
use std::{env, fs, process};
use std::path::Path;

//use pose::get_services_names;
use crate::lib::get_services_names;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(1);
    });
    let yaml = fs::read_to_string(&config.filename).unwrap_or_else(|err| {
        eprintln!("Error reading file: {err}");
        process::exit(2);
    });
    let map = serde_yaml::from_str(&yaml).unwrap_or_else(|err| {
        eprintln!("Error parsing YAML file: {err}");
        process::exit(3);
    });
    get_services_names(&map).iter().for_each(|service| println!("{}", service));
}

struct Config {
    filename: String,
}

impl Config {
    fn build(args: &[String]) -> Result<Config, &'static str> {
        let filename = if args.len() <= 1 {
            if Path::new("docker-compose.yaml").exists() {
                "docker-compose.yaml"
            } else {
                "docker-compose.yml"
            }
        } else {
            &args[1]
        };
        if Path::new(&filename).exists() {
            Ok(Config { filename: String::from(filename) })
        } else {
            Err("No such file")
        }
    }
}