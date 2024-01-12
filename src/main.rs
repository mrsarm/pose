//! `pose` is a command line tool to play with 🐳 Docker Compose files.

use clap::{Parser, Subcommand, ValueEnum};
use std::{fs, process};

//mod lib;
//use crate::lib::{ComposeYaml, get_compose_filename};
use docker_pose::{get_compose_filename, ComposeYaml};

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
            let yaml_content = fs::read_to_string(filename).unwrap_or_else(|err| {
                eprintln!("Error reading compose file: {err}");
                process::exit(11);
            });
            let compose = ComposeYaml::new(&yaml_content).unwrap_or_else(|err| {
                if err.to_string().starts_with("invalid type") {
                    eprintln!("Error parsing compose YAML file: invalid content");
                    process::exit(13);
                }
                eprintln!("Error parsing YAML file: {err}");
                process::exit(14);
            });
            let root_element = object.to_string().to_lowercase();
            let el_iter = compose.get_root_element_names(&root_element).into_iter();
            match pretty {
                Formats::Full => el_iter.for_each(|service| println!("{}", service)),
                Formats::Oneline => println!("{}", el_iter.collect::<Vec<&str>>().join(" ")),
            }
        }
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
    /// List objects found in the compose file: services, volumes, ...
    List {
        #[command(subcommand)]
        object: Objects,

        #[arg(short, long)]
        filename: Option<String>,

        #[arg(short, long, value_enum, default_value_t = Formats::Full, value_name = "FORMAT")]
        pretty: Formats,
    },
    //TODO more coming soon...
}

#[derive(Subcommand, strum_macros::Display)]
enum Objects {
    /// List services
    Services,
    /// List volumes
    Volumes,
    /// List networks
    Networks,
    /// List configs
    Configs,
    /// List secrets
    Secrets,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, strum_macros::Display)]
enum Formats {
    Full,
    Oneline,
}
