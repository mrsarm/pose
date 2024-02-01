//! `pose` is a command line tool to play with 🐳 Docker Compose files.

use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use serde_yaml::Mapping;
use std::vec::IntoIter;
use std::{fs, process};

//mod lib;
//use crate::lib::{ComposeYaml, get_compose_filename};
use docker_pose::{get_compose_filename, ComposeYaml, Verbosity};

fn main() {
    let args = Args::parse();
    let verbosity = args.get_verbosity();
    let filename = get_compose_filename(&args.filename, verbosity).unwrap_or_else(|err| {
        eprintln!("{}: {}", "ERROR".red(), err);
        process::exit(10);
    });
    let yaml_content = fs::read_to_string(filename).unwrap_or_else(|err| {
        eprintln!("{}: reading compose file: {}", "ERROR".red(), err);
        process::exit(11);
    });
    let compose = ComposeYaml::new(&yaml_content).unwrap_or_else(|err| {
        if err.to_string().starts_with("invalid type") {
            eprintln!(
                "{}: parsing compose YAML file: invalid content",
                "ERROR".red()
            );
            process::exit(13);
        }
        eprintln!("{}: parsing YAML file: {}", "ERROR".red(), err);
        process::exit(14);
    });
    match args.command {
        Commands::List { object, pretty } => match object {
            Objects::Envs { service } => {
                let serv = get_service(&compose, &service);
                let envs_op = compose.get_service_envs(serv);
                if let Some(envs) = envs_op {
                    envs.iter().for_each(|env| println!("{}", env));
                }
            }
            Objects::Depends { service } => {
                let serv = get_service(&compose, &service);
                let deps_op = compose.get_service_depends_on(serv);
                if let Some(envs) = deps_op {
                    envs.iter().for_each(|env| println!("{}", env));
                }
            }
            Objects::Images | Objects::Profiles => {
                let op = if object == Objects::Profiles {
                    compose.get_profiles_names()
                } else {
                    compose.get_images()
                };
                match op {
                    None => {
                        eprintln!("{}: No services section found", "ERROR".red());
                        process::exit(15);
                    }
                    Some(profiles) => {
                        print_names(profiles.into_iter(), pretty);
                    }
                }
            }
            Objects::Services
            | Objects::Volumes
            | Objects::Networks
            | Objects::Configs
            | Objects::Secrets => {
                let root_element = object.to_string().to_lowercase();
                let el_iter = compose.get_root_element_names(&root_element).into_iter();
                print_names(el_iter, pretty);
            }
        },
    }
}

fn print_names(iter: IntoIter<&str>, pretty: Formats) {
    match pretty {
        Formats::Full => iter.for_each(|service| println!("{}", service)),
        Formats::Oneline => println!("{}", iter.collect::<Vec<&str>>().join(" ")),
    }
}

fn get_service<'a>(compose: &'a ComposeYaml, service_name: &str) -> &'a Mapping {
    let service = compose.get_service(service_name);
    match service {
        None => {
            eprintln!("{}: No such service found: {}", "ERROR".red(), service_name);
            process::exit(16);
        }
        Some(serv) => serv,
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long)]
    filename: Option<String>,

    /// Increase verbosity
    #[arg(long, conflicts_with = "quiet")]
    verbose: bool,

    /// Only display relevant information or errors
    #[arg(long, short, conflicts_with = "verbose")]
    quiet: bool,
}

impl Args {
    pub fn get_verbosity(&self) -> Verbosity {
        match self.verbose {
            true => Verbosity::Verbose,
            false => match self.quiet {
                true => Verbosity::Quiet,
                false => Verbosity::Info,
            },
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// List objects found in the compose file: services, volumes, ...
    List {
        #[command(subcommand)]
        object: Objects,

        #[arg(short, long, value_enum, default_value_t = Formats::Full, value_name = "FORMAT")]
        pretty: Formats,
    },
    //TODO more coming soon...
}

#[derive(Subcommand, strum_macros::Display, PartialEq)]
enum Objects {
    /// List services
    Services,
    /// List images
    Images,
    /// List service's depends_on
    Depends { service: String },
    /// List volumes
    Volumes,
    /// List networks
    Networks,
    /// List configs
    Configs,
    /// List secrets
    Secrets,
    /// List profiles
    Profiles,
    /// List service's environment variables
    Envs { service: String },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, strum_macros::Display)]
enum Formats {
    Full,
    Oneline,
}
