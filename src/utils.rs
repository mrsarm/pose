use colored::Colorize;
use regex::Regex;
use std::process;

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

/// Get the regex value (or None), or exit if the filter
/// passed doesn't star with "regex=" prefix.
pub fn unwrap_filter_regex(filter: Option<&str>) -> Option<Regex> {
    filter.as_ref().map(|f| {
        if let Some(val) = f.strip_prefix("regex=") {
            Regex::new(val).unwrap_or_else(|e| {
                eprintln!(
                    "{}: invalid regex expression '{}' in filter - {}",
                    "ERROR".red(),
                    val.yellow(),
                    e
                );
                process::exit(2);
            })
        } else {
            eprintln!(
                "{}: wrong filter '{}', only '{}' filter supported",
                "ERROR".red(),
                f.yellow(),
                "regex=".yellow()
            );
            process::exit(2);
        }
    })
}
