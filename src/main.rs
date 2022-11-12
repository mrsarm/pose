use clap::{Parser, Subcommand, ValueEnum};
use std::{fs, process};
use std::path::Path;
use strum_macros;

//mod lib;
//use crate::lib::ComposeYaml;
use pose::ComposeYaml;

fn main() {
    let args = Args::parse();
    match args.command {
        Commands::List {
            filename,
            object,
            pretty,
        } => {
            let filename = get_compose_filename(&filename).unwrap_or_else(|err| {
                eprintln!("{err}");
                process::exit(10);
            });
            let yaml_content = fs::read_to_string(&filename).unwrap_or_else(|err| {
                eprintln!("Error reading file: {err}");
                process::exit(11);
            });
            let compose = ComposeYaml::new(&yaml_content).unwrap_or_else(|err| {
                if err.to_string().starts_with("invalid type") {
                    eprintln!("Error parsing YAML file: invalid file");
                    process::exit(13);
                }
                eprintln!("Error parsing YAML file: {err}");
                process::exit(14);
            });
            let root_element = object.to_string().to_lowercase();
            let el_iter = compose.get_root_element_names(&root_element ).into_iter();
            match pretty {
                Formats::FULL    => el_iter.for_each(|service| println!("{}", service)),
                Formats::ONELINE => println!("{}", el_iter.collect::<Vec<&str>>().join(" ")),
            }
        },
    }
}


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List {
        #[command(subcommand)]
        object: Objects,

        #[arg(short, long)]
        filename: Option<String>,

        #[arg(short, long, value_enum, default_value_t = Formats::FULL, value_name = "FORMAT")]
        pretty: Formats,
    },
    //TODO more coming soon...
}

#[derive(Subcommand, strum_macros::Display)]
enum Objects {
    Services,
    Volumes,
    Networks,
    Configs,
    Secrets,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, strum_macros::Display)]
enum Formats {
    FULL,
    ONELINE,
}

fn get_compose_filename(filename: &Option<String>) -> Result<String, &str> {
    let name = match filename {
        Some(name) => name,
        None =>
            if Path::new("compose.yaml").exists() {
                "compose.yaml"
            } else if Path::new("compose.yml").exists() {
                "compose.yml"
            } else if Path::new("docker-compose.yaml").exists() {
                "docker-compose.yaml"
            } else {
                "docker-compose.yml"
            }
    };
    if Path::new(&name).exists() {
        Ok(String::from(name))
    } else {
        Err("No such file")
    }
}
