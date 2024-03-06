/// Methods to perform command-line tool calls.
/// Used by pose to make calls to the `docker` command
/// and the `git` command.
use crate::Verbosity;
use colored::Colorize;
use std::io::Write;
use std::process::{Command, Output, Stdio};
use std::{io, process};

/// get a string that should be identical to a command-line tool
/// call made by `std::process::Command`.
///
/// Used by ``cmd_call` for debugging.
pub fn cmd_call_to_string(bin: &str, args: &[&str]) -> String {
    format!(
        "{} {}",
        bin,
        args.iter()
            .map(|s| {
                if s.contains(' ') {
                    // TODO better escaping
                    format!("\"{s}\"")
                } else {
                    s.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    )
}

pub fn cmd_call(
    bin: &str,
    args: &[&str],
    output_stdout: bool,
    output_stderr: bool,
    verbosity: &Verbosity,
) -> io::Result<Output> {
    if matches!(verbosity, Verbosity::Verbose) {
        eprintln!("{}: {}", "DEBUG".green(), cmd_call_to_string(bin, args));
    }
    let mut binding = Command::new(bin);
    let mut command = binding.args(args);
    command = command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let output = command.output()?; // an error not from the command but trying to execute it
    if output_stdout {
        cmd_write_stdout(bin, &output.stdout);
    }
    if output_stderr {
        cmd_write_stderr(bin, &output.stderr);
    }
    Ok(output)
}

pub fn cmd_write_stderr(bin: &str, stderr: &[u8]) {
    io::stderr().write_all(stderr).unwrap_or_else(|e| {
        eprintln!("{}: writing {} stderr: {}", "ERROR".red(), bin, e);
        process::exit(151);
    });
}

pub fn cmd_write_stdout(bin: &str, stdout: &[u8]) {
    io::stdout().write_all(stdout).unwrap_or_else(|e| {
        eprintln!("{}: writing {} stdout: {}", "ERROR".red(), bin, e);
        process::exit(151);
    });
}

pub fn cmd_exit_code(bin: &str, output: &Output) -> i32 {
    output.status.code().unwrap_or_else(|| {
        eprintln!("{}: {} process terminated by signal", "ERROR".red(), bin);
        process::exit(10)
    })
}

/// Get the string from the `output` that was generated
/// by a call to `bin bin_cmd`. If was not successful
/// print the error and exit. If `quiet` is `false`
/// also print any warning detected (stderr output).
pub fn cmd_get_success_output_or_fail(
    bin: &str,
    bin_cmd: &str,
    output: Output,
    quiet: bool,
) -> String {
    match output.status.success() {
        true => {
            // success !
            if !quiet && !output.stderr.is_empty() {
                // although, there may be warnings sent to the stderr
                eprintln!(
                    "{}: the following are warnings from {}:",
                    "WARN".yellow(),
                    bin_cmd
                );
                cmd_write_stderr(bin, &output.stderr);
            }
            String::from_utf8(output.stdout).unwrap_or_else(|e| {
                eprintln!(
                    "{}: deserializing {} {} output: {}",
                    "ERROR".red(),
                    bin,
                    bin_cmd,
                    e,
                );
                process::exit(17);
            })
        }
        false => {
            eprintln!("{}: calling {}", "ERROR".red(), bin_cmd);
            cmd_write_stderr(bin, &output.stderr);
            process::exit(cmd_exit_code(bin, &output));
        }
    }
}
