use crate::verbose::Verbosity;
use colored::Colorize;
use std::env::var;
use std::io::Write;
use std::process::{Command, Output, Stdio};
use std::{io, process};

pub struct DockerCommand {
    pub docker_bin: String,
    pub verbosity: Verbosity,
}

impl DockerCommand {
    pub fn new(verbosity: Verbosity) -> Self {
        Self {
            docker_bin: var("DOCKER_BIN").unwrap_or("docker".to_string()),
            verbosity,
        }
    }

    pub fn call_to_string(&self, args: &[&str]) -> String {
        format!(
            "{} {}",
            self.docker_bin,
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

    pub fn call_cmd(
        &self,
        args: &[&str],
        output_stdout: bool,
        output_stderr: bool,
    ) -> io::Result<Output> {
        if matches!(self.verbosity, Verbosity::Verbose) {
            eprintln!("{}: {}", "DEBUG".green(), self.call_to_string(args));
        }
        let mut binding = Command::new(&self.docker_bin);
        let mut command = binding.args(args);
        command = command.stdout(Stdio::piped()).stderr(Stdio::piped());
        let output = command.output()?; // an error not from the command but trying to execute it
        if output_stdout {
            self.write_stdout(&output.stdout);
        }
        if output_stderr {
            self.write_stderr(&output.stderr);
        }
        Ok(output)
    }

    pub fn call_compose_cmd(
        &self,
        cmd: &str,
        filenames: &[&str],
        args: &[&str],
        output_stdout: bool,
        output_stderr: bool,
    ) -> io::Result<Output> {
        let mut cmd_args = vec!["compose"];
        for filename in filenames {
            cmd_args.push("-f");
            cmd_args.push(filename);
        }
        for arg in args {
            cmd_args.push(arg);
        }
        cmd_args.push(cmd);
        self.call_cmd(&cmd_args, output_stdout, output_stderr)
    }

    pub fn call_compose_config(
        &self,
        filenames: &[&str],
        output_stdout: bool,
        output_stderr: bool,
    ) -> io::Result<Output> {
        self.call_compose_cmd(
            "config",
            filenames,
            &Vec::default(),
            output_stdout,
            output_stderr,
        )
    }

    pub fn write_stderr(&self, stderr: &[u8]) {
        io::stderr().write_all(stderr).unwrap_or_else(|e| {
            eprintln!(
                "{}: writing {} stderr: {}",
                "ERROR".red(),
                self.docker_bin,
                e
            );
            process::exit(151);
        });
    }

    pub fn write_stdout(&self, stdout: &[u8]) {
        io::stdout().write_all(stdout).unwrap_or_else(|e| {
            eprintln!(
                "{}: writing {} stdout: {}",
                "ERROR".red(),
                self.docker_bin,
                e
            );
            process::exit(151);
        });
    }

    pub fn exit_code(&self, output: &Output) -> i32 {
        output.status.code().unwrap_or_else(|| {
            eprintln!(
                "{}: {} process terminated by signal",
                "ERROR".red(),
                self.docker_bin
            );
            process::exit(10)
        })
    }
}
