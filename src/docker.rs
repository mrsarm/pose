use crate::verbose::Verbosity;
use crate::{cmd_call, cmd_call_to_string, cmd_exit_code, cmd_write_stderr, cmd_write_stdout};

use std::env::var;
use std::io;
use std::process::Output;

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
        cmd_call_to_string(&self.docker_bin, args)
    }

    pub fn call_cmd(
        &self,
        args: &[&str],
        output_stdout: bool,
        output_stderr: bool,
    ) -> io::Result<Output> {
        cmd_call(
            &self.docker_bin,
            args,
            output_stdout,
            output_stderr,
            &self.verbosity,
        )
    }

    pub fn call_compose_cmd(
        &self,
        cmd: &str,
        filenames: &[&str],
        args: &[&str],
        cmd_args: &[&str],
        output_stdout: bool,
        output_stderr: bool,
    ) -> io::Result<Output> {
        let mut docker_args = vec!["compose"];
        for filename in filenames {
            docker_args.push("-f");
            docker_args.push(filename);
        }
        for arg in args {
            docker_args.push(arg);
        }
        docker_args.push(cmd);
        for arg in cmd_args {
            docker_args.push(arg);
        }
        self.call_cmd(&docker_args, output_stdout, output_stderr)
    }

    pub fn call_compose_config(
        &self,
        filenames: &[&str],
        no_consistency: bool,
        no_interpolate: bool,
        no_normalize: bool,
        output_stdout: bool,
        output_stderr: bool,
    ) -> io::Result<Output> {
        let mut cmd_args = Vec::new();
        if no_consistency {
            cmd_args.push("--no-consistency");
        }
        if no_interpolate {
            cmd_args.push("--no-interpolate");
        }
        if no_normalize {
            cmd_args.push("--no-normalize");
        }
        self.call_compose_cmd(
            "config",
            filenames,
            &Vec::default(),
            &cmd_args,
            output_stdout,
            output_stderr,
        )
    }

    pub fn get_manifest_inspect(&self, image: &str) -> io::Result<Output> {
        self.call_cmd(&["manifest", "inspect", "--insecure", image], false, false)
    }

    pub fn get_image_inspect(&self, image: &str) -> io::Result<Output> {
        self.call_cmd(&["image", "inspect", image], false, false)
    }

    pub fn write_stderr(&self, stderr: &[u8]) {
        cmd_write_stderr(&self.docker_bin, stderr);
    }

    pub fn write_stdout(&self, stdout: &[u8]) {
        cmd_write_stdout(&self.docker_bin, stdout);
    }

    pub fn exit_code(&self, output: &Output) -> i32 {
        cmd_exit_code(&self.docker_bin, output)
    }
}
