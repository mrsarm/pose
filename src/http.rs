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
) {
    let mut url = url.to_string();
    let parsed_url = match Url::parse(&url) {
        Ok(r) => r,
        Err(e) => {
            if let Some(script) = script {
                url = url.replace(&script.0, &script.1);
                Url::parse(&url).unwrap_or_else(|e| {
                    eprintln!("{}: invalid URL and replace script - {}", "ERROR".red(), e);
                    process::exit(3)
                })
            } else {
                eprintln!("{}: invalid URL - {}", "ERROR".red(), e);
                process::exit(3);
            }
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
    match agent.get(&url).call() {
        Ok(resp) => {
            save(resp, path, output);
        }
        Err(Error::Status(code, response)) => {
            if response.status() != 404 {
                eprintln!(
                    "{}: {} {} {}",
                    "ERROR".red(),
                    response.http_version(),
                    code,
                    response.status_text()
                );
                eprintln!("{}", response.into_string().unwrap_or("".to_string()));
                process::exit(1);
            } else {
                // TODO use script
            }
        }
        Err(e) => {
            eprintln!("{}: {}", "ERROR".red(), e);
            process::exit(2);
        }
    }
}

fn save(resp: Response, path: &Path, output: &Option<String>) {
    let filename = if let Some(filename) = output {
        filename
    } else {
        path.file_name().unwrap().to_str().unwrap()
    };
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

/// Return a vector with each string equals to the template string, but replacing on each one
/// the left part of one of the replacers with the right side of the replacer expression.
/// Each replacer has to have the form "string-to-replace:replacement".
///
/// ```
/// use docker_pose::replace_all;
///
/// let replaces = replace_all(
///     "https://github.com/mrsarm/pose/archive/refs/tags/0.3.0.zip",
///     vec!["0.3.0:0.4.0".to_string(), "0.3.0:latest".to_string()].as_ref(),
/// );
/// assert_eq!(
///     replaces.unwrap_or_default(),
///     vec![
///         "https://github.com/mrsarm/pose/archive/refs/tags/0.4.0.zip".to_string(),
///         "https://github.com/mrsarm/pose/archive/refs/tags/latest.zip".to_string(),
///     ],
/// );
///
/// let replaces = replace_all(
///     "-",
///     vec!["-:something".to_string(), "-:totally".to_string(), "-:new".to_string()].as_ref()
/// );
/// assert_eq!(
///     replaces.unwrap_or_default(),
///     vec!["something".to_string(), "totally".to_string(), "new".to_string()],
/// );
///
/// let replaces = replace_all(
///     "pose-0.3.zip",
///     vec!["0.3:0.4".to_string(), "missing-separator".to_string()].as_ref(),
/// );
/// assert_eq!(
///     replaces,
///     Err("Expression \"missing-separator\" doesn't have the separator symbol `:'".to_string())
/// );
///
/// let replaces = replace_all(
///     "hard-to-replace",
///     vec!["not-there:something".to_string()].as_ref()
/// );
/// assert_eq!(
///     replaces,
///     Err("Left part of the expression \"not-there:something\" not found in \"hard-to-replace\"".to_string())
/// );
/// ```
pub fn replace_all(template: &str, replacers: &Vec<String>) -> Result<Vec<String>, String> {
    let mut v: Vec<String> = Vec::with_capacity(replacers.len());
    for replacer in replacers {
        let mut split = replacer.split(':');
        let left = split.next();
        let right = split.next();
        if let Some(right_text) = right {
            let left_text = left.unwrap();
            if !template.contains(left_text) {
                return Err(format!(
                    "Left part of the expression \"{}\" not found in \"{}\"",
                    replacer, template,
                ));
            }
            v.push(template.replace(left_text, right_text))
        } else {
            return Err(format!(
                "Expression \"{}\" doesn't have the separator symbol `:'",
                replacer
            ));
        }
    }
    Ok(v)
}
