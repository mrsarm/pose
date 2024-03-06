use crate::{get_compose_filename, ComposeYaml, Formats, Verbosity};
use colored::Colorize;
use regex::Regex;
use serde_yaml::Mapping;
use std::cmp::min;
use std::vec::IntoIter;
use std::{fs, process};

/// Get the tag value (or None), or exit if the filter
/// passed doesn't star with "tag=" prefix.
pub fn unwrap_filter_tag(filter: Option<&str>) -> Option<&str> {
    filter.as_ref().map(|f| {
        if let Some(val) = f.strip_prefix("tag=") {
            val
        } else {
            eprintln!(
                "{}: wrong filter '{}', only '{}' filter supported",
                "ERROR".red(),
                f.yellow(),
                "tag=".yellow()
            );
            process::exit(2);
        }
    })
}

/// Get the regex value expressed in &str, or exit if the filter
/// passed doesn't star with "regex=" or "regex!=" prefixes.
/// When expression has "=" the bool is true, when is "!="
/// the bool is false.
/// If the option passed is None, this method returns None as well.
pub fn unwrap_filter_regex(filter: Option<&str>) -> Option<(Regex, bool)> {
    filter.as_ref().map(|f| {
        if let Some(val) = f.strip_prefix("regex=") {
            let regex = Regex::new(val).unwrap_or_else(|e| {
                invalid_regex_exit(e, val);
            });
            return (regex, true);
        }
        if let Some(val) = f.strip_prefix("regex!=") {
            let regex = Regex::new(val).unwrap_or_else(|e| {
                invalid_regex_exit(e, val);
            });
            return (regex, false);
        }
        eprintln!(
            "{}: wrong filter '{}', only '{}' or '{}' filters are supported",
            "ERROR".red(),
            f.yellow(),
            "regex=".yellow(),
            "regex!=".yellow()
        );
        process::exit(2);
    })
}

fn invalid_regex_exit(e: regex::Error, val: &str) -> ! {
    eprintln!(
        "{}: invalid regex expression '{}' in filter - {}",
        "ERROR".red(),
        val.yellow(),
        e
    );
    process::exit(2);
}

pub fn print_names(iter: IntoIter<&str>, pretty: Formats) {
    match pretty {
        Formats::Full => iter.for_each(|service| println!("{}", service)),
        Formats::Oneline => println!("{}", iter.collect::<Vec<&str>>().join(" ")),
    }
}

pub fn get_service<'a>(compose: &'a ComposeYaml, service_name: &str) -> &'a Mapping {
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

/// Get a slug version of the text compatible with
/// a tag name to be published in a docker registry, with
/// only number, letters, the symbol "-" or the symbol ".",
/// and no more than 63 characters long, all in lowercase.
pub fn get_slug(input: &str) -> String {
    let text = input.trim();
    let text = text.to_lowercase();
    let len = min(text.len(), 63);
    let mut result = String::with_capacity(len);

    for c in text.chars() {
        if result.len() >= len {
            break;
        }
        if c.is_ascii_alphanumeric() || c == '.' {
            result.push(c);
        } else {
            result.push('-');
        }
    }
    result
}
