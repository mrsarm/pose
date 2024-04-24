#[macro_use]
extern crate lazy_static;

mod args;
mod cmd;
mod docker;
mod git;
mod http;
mod parse;
mod utils;
mod verbose;

pub use args::{Args, Commands, Formats, Objects};
pub use cmd::{
    cmd_call, cmd_call_to_string, cmd_exit_code, cmd_get_success_output_or_fail, cmd_write_stderr,
    cmd_write_stdout,
};
pub use docker::DockerCommand;
pub use git::GitCommand;
pub use http::get_and_save;
pub use parse::{
    get_compose_filename, header, positive_less_than_32, string_no_empty, string_script,
    ComposeYaml, ReplaceTag,
};
pub use utils::{
    get_service, get_slug, get_yml_content, print_names, unwrap_filter_regex, unwrap_filter_tag,
};
pub use verbose::Verbosity;
