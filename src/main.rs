//! `pose` is a command line tool to play with 🐳 Docker Compose files.

use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use serde_yaml::Mapping;
use std::vec::IntoIter;
use std::{fs, process};

//mod lib;
//use crate::lib::{ComposeYaml, get_compose_filename};
use docker_pose::{
    get_compose_filename, unwrap_filter_regex, unwrap_filter_tag, ComposeYaml, DockerCommand,
    RemoteTag, Verbosity,
};

fn main() {
    let args = Args::parse();
    let verbosity = args.get_verbosity();
    if args.filenames.len() > 1 && args.no_docker {
        eprintln!(
            "{}: multiple '{}' arguments cannot be used with '{}'",
            "ERROR".red(),
            "--file".yellow(),
            "--no-docker".yellow()
        );
        process::exit(2);
    }
    let yaml_content = match args.no_docker {
        true => get_yml_content(args.filenames.first().map(AsRef::as_ref), verbosity.clone()),
        false => {
            let command = DockerCommand::new(verbosity.clone());
            let result_output = command.call_compose_config(
                &args.filenames.iter().map(AsRef::as_ref).collect::<Vec<_>>(),
                false,
                false,
            );
            match result_output {
                Ok(output) => {
                    // docker was successfully called by pose, but docker compose
                    // could either succeed or fail executing its task
                    match output.status.success() {
                        true => {
                            // success !
                            if !args.quiet && !output.stderr.is_empty() {
                                // although, there may be warnings sent to the stderr
                                eprintln!(
                                    "{}: the following are warnings from compose:",
                                    "WARN".yellow()
                                );
                                command.write_stderr(&output.stderr);
                            }
                            String::from_utf8(output.stdout).unwrap_or_else(|e| {
                                eprintln!(
                                    "{}: deserializing {} compose output: {}",
                                    "ERROR".red(),
                                    command.docker_bin,
                                    e,
                                );
                                process::exit(17);
                            })
                        }
                        false => {
                            eprintln!("{}: calling compose", "ERROR".red());
                            command.write_stderr(&output.stderr);
                            process::exit(command.exit_code(&output));
                        }
                    }
                }
                Err(e) => {
                    // docker couldn't be called by pose or the OS
                    eprintln!("{}: calling compose: {}", "ERROR".red(), e);
                    eprintln!(
                        "{}: parsing will be executed without compose",
                        "WARN".yellow()
                    );
                    get_yml_content(args.filenames.first().map(AsRef::as_ref), verbosity.clone())
                }
            }
        }
    };
    let compose = ComposeYaml::new(&yaml_content).unwrap_or_else(|err| {
        if err.to_string().starts_with("invalid type") {
            eprintln!(
                "{}: parsing compose YAML file: invalid content",
                "ERROR".red()
            );
            process::exit(13);
        }
        eprintln!("{}: parsing YAML file: {}", "ERROR".red(), err);
        process::exit(15);
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
            Objects::Profiles => {
                let op = compose.get_profiles_names();
                match op {
                    None => {
                        eprintln!("{}: No profiles section found", "ERROR".red());
                        process::exit(15);
                    }
                    Some(profiles) => {
                        print_names(profiles.into_iter(), pretty);
                    }
                }
            }
            Objects::Images {
                filter,
                remote_tag,
                remote_tag_filter,
                ignore_unauthorized,
            } => {
                let tag = unwrap_filter_tag(filter.as_deref());
                let regex = unwrap_filter_regex(remote_tag_filter.as_deref());
                let remote_tag = remote_tag.map(|tag| RemoteTag {
                    ignore_unauthorized,
                    remote_tag: tag,
                    remote_tag_filter: regex,
                    verbosity: verbosity.clone(),
                });
                let op = compose.get_images(tag, remote_tag);
                match op {
                    None => {
                        eprintln!("{}: No services section found", "ERROR".red());
                        process::exit(15);
                    }
                    Some(images) => {
                        let images_list = images.iter().map(|i| i.as_str()).collect::<Vec<_>>();
                        print_names(images_list.into_iter(), pretty);
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

pub fn get_yml_content(filename: Option<&str>, verbosity: Verbosity) -> String {
    let filename = get_compose_filename(filename, verbosity).unwrap_or_else(|err| {
        eprintln!("{}: {}", "ERROR".red(), err);
        if err.contains("no such file or directory") {
            process::exit(14);
        }
        process::exit(10);
    });
    fs::read_to_string(filename).unwrap_or_else(|err| {
        eprintln!("{}: reading compose file: {}", "ERROR".red(), err);
        process::exit(11);
    })
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long = "file")]
    filenames: Vec<String>,

    /// Increase verbosity
    #[arg(long, conflicts_with = "quiet")]
    verbose: bool,

    /// Only display relevant information or errors
    #[arg(long, short, conflicts_with = "verbose")]
    quiet: bool,

    /// Don't call docker compose to parse compose model
    #[arg(long)]
    no_docker: bool,
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
    Images {
        /// filter by a property, if --remote-tag is used as well,
        /// this filter is applied first, filtering out images that
        /// don't match the filter. Currently only tag=TAG is supported
        #[arg(short, long)]
        filter: Option<String>,
        /// print with remote tag passed instead of the one set in the file
        /// if exists in the docker registry
        #[arg(short, long, value_name = "TAG")]
        remote_tag: Option<String>,
        /// use with --remote-tag to filter which images should be checked
        /// whether the remote tag exists or not, but images that don't match
        /// the filter are not filtered out from the list printed, only
        /// printed with the tag they have in the compose file.
        /// Currently only regex=NAME is supported
        // TODO implement
        #[arg(long, value_name = "NAME", requires("remote_tag"))]
        remote_tag_filter: Option<String>,
        /// ignore unauthorized errors from docker when fetching remote tags info
        #[arg(long, requires("remote_tag"))]
        ignore_unauthorized: bool,
    },
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
