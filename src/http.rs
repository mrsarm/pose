use clap::crate_version;
use colored::Colorize;
use std::fs::File;
use std::path::Path;
use std::time::Duration;
use std::{io, process};
use ureq::{Agent, AgentBuilder, Error};
use url::Url;

pub fn get(url: &str) {
    let parsed_url = match Url::parse(url) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}: invalid URL - {}", "ERROR".red(), e);
            process::exit(3);
        }
    };
    let path = if parsed_url.path() == "/" {
        eprintln!(
            "{}: URL without filename, you have to provide \
            the filename where to store the file with the argument {}",
            "ERROR".red(),
            "-o, --output".yellow()
        );
        process::exit(4); // TODO add -o argument
    } else {
        parsed_url.path()
    };
    let path = Path::new(path);
    let filename = path.file_name().unwrap().to_str().unwrap();
    let agent: Agent = AgentBuilder::new()
        .timeout(Duration::from_secs(5)) // TODO add timeout argument
        .user_agent(format!("pose/{}", crate_version!()).as_str())
        .build();
    match agent.get(url).call() {
        Ok(resp) => {
            let mut content = resp.into_reader();
            let mut file = File::create(filename).unwrap_or_else(|e| {
                eprintln!(
                    "{}: creating file '{}' - {}",
                    "ERROR".red(),
                    filename.yellow(),
                    e
                );
                process::exit(5);
            });
            io::copy(&mut content, &mut file).unwrap_or_else(|e| {
                eprintln!(
                    "{}: writing output to file '{}': {}",
                    "ERROR".red(),
                    filename.yellow(),
                    e
                );
                process::exit(6);
            });
        }
        Err(Error::Status(code, response)) => {
            eprintln!(
                "{}: {} {} {}",
                "ERROR".red(),
                response.http_version(),
                code,
                response.status_text()
            );
            eprintln!("{}", response.into_string().unwrap_or("".to_string()));
            process::exit(1);
        }
        Err(e) => {
            eprintln!("{}: {}", "ERROR".red(), e);
            process::exit(2);
        }
    }
}
