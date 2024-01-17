//! `pose` is a command line tool to play with 🐳 Docker Compose files.

use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use serde_yaml::Mapping;
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
            if let Objects::Envs { service } = object {
                let serv = get_service(&compose, &service);
                let envs_op = compose.get_service_envs(serv);
                if let Some(envs) = envs_op {
                    envs.iter().for_each(|env| println!("{}", env));
                }
                return;
            }
            let root_element = object.to_string().to_lowercase();
            let el_iter = compose.get_root_element_names(&root_element).into_iter();
            match pretty {
                Formats::Full => el_iter.for_each(|service| println!("{}", service)),
                Formats::Oneline => println!("{}", el_iter.collect::<Vec<&str>>().join(" ")),
            }
        }
    }
}

pub fn get_service<'a>(compose: &'a ComposeYaml, service_name: &str) -> &'a Mapping {
    let service = compose.get_service(service_name);
    match service {
        None => {
            eprintln!("{}: No such service found: {}", "ERROR".red(), service_name);
            process::exit(15);
        }
        Some(serv) => serv,
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
    /// List service's environment variables
    Envs { service: String },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, strum_macros::Display)]
enum Formats {
    Full,
    Oneline,
}
