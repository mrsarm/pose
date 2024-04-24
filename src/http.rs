use crate::Verbosity;
use clap::crate_version;
use colored::Colorize;
use std::fs::File;
use std::path::Path;
use std::time::Duration;
use std::{io, process};
use ureq::{Agent, AgentBuilder, Error, Response};
use url::Url;

pub fn get_and_save(
    url: &str,
    script: &Option<(String, String)>,
    output: &Option<String>,
    timeout_connect_secs: u16,
    max_time: u16,
    verbosity: Verbosity,
) {
    let mut url = url.to_string();
    let parsed_url = match Url::parse(&url) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}: invalid URL - {}", "ERROR".red(), e);
            process::exit(3);
        }
    };
    let path = if parsed_url.path() == "/" {
        output.as_ref().unwrap_or_else(|| {
            eprintln!(
                "{}: URL without filename, you have to provide \
                the filename where to store the file with the argument {}",
                "ERROR".red(),
                "-o, --output".yellow()
            );
            process::exit(4);
        })
    } else {
        parsed_url.path()
    };
    let path = Path::new(path);
    let agent: Agent = AgentBuilder::new()
        .timeout_connect(Duration::from_secs(timeout_connect_secs.into()))
        .timeout(Duration::from_secs(max_time.into()))
        .user_agent(format!("pose/{}", crate_version!()).as_str())
        .build();
    let mut result = _get_and_save(&url, output, path, &agent, verbosity.clone());
    if !result {
        if let Some(script) = script {
            if !url.contains(&script.0) {
                eprintln!(
                    "{}: the left part of the script '{}' is not part of the URL",
                    "ERROR".red(),
                    script.0.yellow()
                );
                process::exit(10);
            }
            url = url.replace(&script.0, &script.1);
            result = _get_and_save(&url, output, path, &agent, verbosity.clone());
        }
    }
    if !result {
        eprintln!("{}: Download failed", "ERROR".red());
        process::exit(1);
    }
}

fn _get_and_save(
    url: &str,
    output: &Option<String>,
    path: &Path,
    agent: &Agent,
    verbosity: Verbosity,
) -> bool {
    if !matches!(verbosity, Verbosity::Quiet) {
        eprint!("{}: Downloading {} ... ", "DEBUG".green(), url);
    }
    match agent.get(url).call() {
        Ok(resp) => {
            if !matches!(verbosity, Verbosity::Quiet) {
                eprintln!("{}", "found".green());
            }
            save(resp, path, output, verbosity.clone());
            true
        }
        Err(Error::Status(code, response)) => {
            if response.status() != 404 {
                if !matches!(verbosity, Verbosity::Quiet) {
                    eprintln!("{}", "failed".red())
                }
                eprintln!(
                    "{}: {} {} {}",
                    "ERROR".red(),
                    response.http_version(),
                    code,
                    response.status_text()
                );
                eprintln!("{}", response.into_string().unwrap_or("".to_string()));
                process::exit(5);
            } else {
                if !matches!(verbosity, Verbosity::Quiet) {
                    eprintln!("{}", "not found".purple());
                }
                false
            }
        }
        Err(e) => {
            if !matches!(verbosity, Verbosity::Quiet) {
                eprintln!("{}", "failed".red())
            }
            eprintln!("{}: {}", "ERROR".red(), e);
            process::exit(7);
        }
    }
}

fn save(resp: Response, path: &Path, output: &Option<String>, verbosity: Verbosity) {
    let filename = if let Some(filename) = output {
        if !matches!(verbosity, Verbosity::Quiet) {
            eprint!(
                "{}: Saving downloaded file as {} ... ",
                "DEBUG".green(),
                filename.yellow()
            );
        }
        filename
    } else {
        path.file_name().unwrap().to_str().unwrap()
    };
    let mut content = resp.into_reader();
    let mut file = File::create(filename).unwrap_or_else(|e| {
        if !matches!(verbosity, Verbosity::Quiet) {
            eprintln!("{}", "failed".red())
        }
        eprintln!(
            "{}: creating file '{}' - {}",
            "ERROR".red(),
            filename.yellow(),
            e
        );
        process::exit(5);
    });
    io::copy(&mut content, &mut file).unwrap_or_else(|e| {
        if !matches!(verbosity, Verbosity::Quiet) {
            eprintln!("{}", "failed".red());
        }
        eprintln!(
            "{}: writing output to file '{}': {}",
            "ERROR".red(),
            filename.yellow(),
            e
        );
        process::exit(6);
    });
    if !matches!(verbosity, Verbosity::Quiet) && output.is_some() {
        eprintln!("{}", "done".green());
    }
}
