/// Types to parse the command line arguments with the clap crate.
use crate::Verbosity;
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long = "file")]
    pub filenames: Vec<String>,

    /// Increase verbosity
    #[arg(long, conflicts_with = "quiet")]
    pub verbose: bool,

    /// Only display relevant information or errors
    #[arg(long, short, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Don't call docker compose to parse compose model
    #[arg(long)]
    pub no_docker: bool,

    /// Don't check model consistency - warning: may produce invalid Compose output
    #[arg(long)]
    pub no_consistency: bool,

    /// Don't interpolate environment variables
    #[arg(long)]
    pub no_interpolate: bool,

    /// Don't normalize compose model
    #[arg(long)]
    pub no_normalize: bool,
}

impl Args {
    pub fn get_verbosity(&self) -> Verbosity {
        match self.verbose {
            true => Verbosity::Verbose,
            false => match self.quiet {
                true => Verbosity::Quiet,
                false => Verbosity::Info,
            },
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// List objects found in the compose file: services, volumes, ...
    List {
        #[command(subcommand)]
        object: Objects,

        #[arg(short, long, value_enum, default_value_t = Formats::Full, value_name = "FORMAT")]
        pretty: Formats,
    },
    /// Parse, resolve and render compose file in canonical format
    Config {
        /// Save to file (default to stdout)
        #[arg(short, long, value_name = "FILE")]
        output: Option<String>,
    },
}

#[derive(Subcommand, strum_macros::Display, PartialEq)]
pub enum Objects {
    /// List services
    Services,
    /// List images
    Images {
        /// filter by a property, if --remote-tag is used as well,
        /// this filter is applied first, filtering out images that
        /// don't match the filter. Currently only tag=TAG is supported
        #[arg(short, long)]
        filter: Option<String>,
        /// print with remote tag passed instead of the one set in the file
        /// if exists in the docker registry
        #[arg(short, long, value_name = "TAG")]
        remote_tag: Option<String>,
        /// use with --remote-tag to filter which images should be checked
        /// whether the remote tag exists or not, but images that don't match
        /// the filter are not filtered out from the list printed, only
        /// printed with the tag they have in the compose file.
        /// Currently only regex=NAME is supported
        #[arg(long, value_name = "FILTER", requires("remote_tag"))]
        remote_tag_filter: Option<String>,
        /// ignore unauthorized errors from docker when fetching remote tags info
        #[arg(long, requires("remote_tag"))]
        ignore_unauthorized: bool,
    },
    /// List service's depends_on
    Depends { service: String },
    /// List volumes
    Volumes,
    /// List networks
    Networks,
    /// List configs
    Configs,
    /// List secrets
    Secrets,
    /// List profiles
    Profiles,
    /// List service's environment variables
    Envs { service: String },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, strum_macros::Display)]
pub enum Formats {
    Full,
    Oneline,
}