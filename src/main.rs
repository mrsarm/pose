//! `pose` is a command line tool to play with 🐳 Docker Compose files.

use clap::Parser;
use colored::*;
use std::{fs, process};

//mod lib;
//use crate::lib::ComposeYaml;
use docker_pose::{
    cmd_get_success_output_or_fail, get_service, get_slug, get_yml_content, print_names,
    unwrap_filter_regex, unwrap_filter_tag, Args, Commands, ComposeYaml, DockerCommand, GitCommand,
    Objects, ReplaceTag, Verbosity,
};

fn main() {
    setup_terminal();
    let args = Args::parse();
    let verbosity = args.get_verbosity();
    if let Commands::Slug { text } = args.command {
        if let Some(t) = text {
            println!("{}", get_slug(&t));
        } else {
            let command = GitCommand::new(verbosity.clone());
            let result_output = command.get_current_branch();
            match result_output {
                Ok(output) => {
                    // git was successfully called by pose, but docker compose
                    // could either succeed or fail executing its task
                    let slug = cmd_get_success_output_or_fail(
                        &command.git_bin,
                        "rev-parse",
                        output,
                        args.quiet,
                    );
                    println!("{}", get_slug(&slug));
                }
                Err(e) => {
                    // git couldn't be called by pose or the OS
                    eprintln!("{}: calling git: {}", "ERROR".red(), e);
                    process::exit(21);
                }
            }
        }
        process::exit(0)
    }
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
                args.no_consistency,
                args.no_interpolate,
                args.no_normalize,
                false,
                false,
            );
            match result_output {
                Ok(output) => {
                    // docker was successfully called by pose, but docker compose
                    // could either succeed or fail executing its task
                    cmd_get_success_output_or_fail(
                        &command.docker_bin,
                        "compose",
                        output,
                        args.quiet,
                    )
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
    let mut compose = ComposeYaml::new(&yaml_content).unwrap_or_else(|err| {
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
                tag_filter,
                ignore_unauthorized,
                remote_progress,
                no_slug,
                threads,
            } => {
                let tag = unwrap_filter_tag(filter.as_deref());
                let regex = unwrap_filter_regex(tag_filter.as_deref());
                let remote_tag = remote_tag.map(|tag| ReplaceTag {
                    ignore_unauthorized,
                    threads,
                    no_slug,
                    remote_tag: tag,
                    tag_filter: regex,
                    verbosity: verbosity.clone(),
                    remote_progress_verbosity: match remote_progress {
                        true => Verbosity::Verbose,
                        false => Verbosity::Quiet,
                    },
                });
                let op = compose.get_images(tag, remote_tag.as_ref());
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
        Commands::Config {
            output,
            remote_tag,
            tag_filter,
            ignore_unauthorized,
            remote_progress,
            no_slug,
            threads,
        } => {
            let regex = unwrap_filter_regex(tag_filter.as_deref());
            let remote_tag = remote_tag.map(|tag| ReplaceTag {
                ignore_unauthorized,
                threads,
                no_slug,
                remote_tag: tag,
                tag_filter: regex,
                verbosity: verbosity.clone(),
                remote_progress_verbosity: match remote_progress {
                    true => Verbosity::Verbose,
                    false => Verbosity::Quiet,
                },
            });
            if let Some(remote_t) = remote_tag {
                compose.update_images_with_remote_tag(&remote_t);
            }
            let result = compose.to_string().unwrap_or_else(|err| {
                eprintln!("{}: {}", "ERROR".red(), err);
                process::exit(20);
            });
            if let Some(file) = output {
                fs::write(&file, result).unwrap_or_else(|e| {
                    eprintln!(
                        "{}: writing output to '{}' file: {}",
                        "ERROR".red(),
                        file.yellow(),
                        e
                    );
                    process::exit(18);
                });
            } else {
                println!("{}", result);
            }
        }
        Commands::Slug { .. } => {
            // This was attended above in the code
        }
    }
}

#[cfg(target_os = "windows")]
fn setup_terminal() {
    control::set_virtual_terminal(true).unwrap();
}

#[cfg(not(target_os = "windows"))]
fn setup_terminal() {
    // nothing is needed in *nix systems
}
