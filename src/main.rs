use clap::{Parser, Subcommand};
use std::{fs, process};
use std::path::Path;

//mod lib;
//use crate::lib::ComposeYaml;
use pose::ComposeYaml;

fn main() {
    let args = Args::parse();
    match args.command {
        Commands::List { filename, object } => {
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
            compose
                .get_root_element_names(&root_element )
                .iter()
                .for_each(|service| println!("{}", service));
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
        #[arg(short, long)]
        filename: Option<String>,

        #[command(subcommand)]
        object: Objects,
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

fn get_compose_filename(filename: &Option<String>) -> Result<String, &str> {
    let name = match filename {
        Some(name) => name,
        None =>
            if Path::new("docker-compose.yaml").exists() {
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
