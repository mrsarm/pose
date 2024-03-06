use crate::Verbosity;
use crate::{cmd_call, cmd_call_to_string};

use std::env::var;
use std::io;
use std::process::Output;

pub struct GitCommand {
    pub git_bin: String,
    pub verbosity: Verbosity,
}

impl GitCommand {
    pub fn new(verbosity: Verbosity) -> Self {
        Self {
            git_bin: var("GIT_BIN").unwrap_or("git".to_string()),
            verbosity,
        }
    }

    pub fn call_to_string(&self, args: &[&str]) -> String {
        cmd_call_to_string(&self.git_bin, args)
    }

    pub fn call_cmd(
        &self,
        args: &[&str],
        output_stdout: bool,
        output_stderr: bool,
    ) -> io::Result<Output> {
        cmd_call(
            &self.git_bin,
            args,
            output_stdout,
            output_stderr,
            &self.verbosity,
        )
    }

    pub fn get_current_branch(&self) -> io::Result<Output> {
        self.call_cmd(&["rev-parse", "--abbrev-ref", "HEAD"], false, false)
    }
}
