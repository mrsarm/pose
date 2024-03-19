/// Types to parse the command line arguments with the clap crate.
use crate::Verbosity;
use clap::{Parser, Subcommand, ValueEnum};
use clap_num::number_range;
use std::cmp::Ord;

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

fn positive_less_than_32(s: &str) -> Result<u8, String> {
    number_range(s, 1, 32)
}

fn string_no_empty(s: &str) -> Result<String, String> {
    if s.is_empty() {
        return Err("must be at least 1 character long".to_string());
    }
    Ok(s.to_string())
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
        /// output image attributes in services with tag passed instead of the one set in the file
        /// if they exist locally or in the remote docker registry
        #[arg(short, long, value_name = "TAG", value_parser = string_no_empty)]
        tag: Option<String>,
        /// use with --tag to filter which images should be checked whether the
        /// tag exists or not locally or remotely.
        /// Currently only regex=EXPR or regex!=EXPR are supported
        #[arg(long, value_name = "FILTER", requires("tag"), value_parser = string_no_empty)]
        tag_filter: Option<String>,
        /// ignore unauthorized errors from docker when fetching remote tags info
        #[arg(long, requires("tag"))]
        ignore_unauthorized: bool,
        /// Don't slugify the value from --tag
        #[arg(long, requires("tag"))]
        no_slug: bool,
        /// only check --tag TAG with the local docker registry
        #[arg(long, requires("tag"))]
        offline: bool,
        /// outputs in stderr the progress of fetching the tags info, similar to --verbose,
        /// but without all the other details --verbose adds
        #[arg(long, requires("tag"))]
        progress: bool,
        /// max number of threads used to fetch remote images info
        #[arg(long, value_name = "NUM", default_value_t = 8, value_parser = positive_less_than_32, requires("tag"))]
        threads: u8,
    },
    /// Outputs a slug version of the text passed, or the slug version of the
    /// current branch.
    ///
    /// It's the same slug used with the --tag value in other commands.
    /// The output is a lowercase version with all no-alphanumeric
    /// characters translated into the "-" symbol, except for the char ".", to make it
    /// compatible with a valid docker tag name.
    Slug {
        /// text to slugify, if not provided the current branch name is used
        #[arg(value_parser = string_no_empty)]
        text: Option<String>,
    },
}

#[derive(Subcommand, strum_macros::Display, PartialEq)]
pub enum Objects {
    /// List services
    Services,
    /// List images
    Images {
        /// filter by a property, if --tag is used as well,
        /// this filter is applied first, filtering out images that
        /// don't match the filter. Currently only tag=TAG is supported
        #[arg(short, long)]
        filter: Option<String>,
        /// print images with the tag passed instead of the one set in the file if they exist
        /// locally or in the remote docker registry
        #[arg(short, long, value_name = "TAG", value_parser = string_no_empty)]
        tag: Option<String>,
        /// use with --tag to filter which images should be checked whether the tag exists
        /// or not, but images that don't match the filter are not filtered out from the list
        /// printed, only printed with the tag they have in the compose file.
        /// Currently only regex=EXPR or regex!=EXPR are supported
        #[arg(long, value_name = "FILTER", requires("tag"), value_parser = string_no_empty)]
        tag_filter: Option<String>,
        /// ignore unauthorized errors from docker when fetching remote tags info
        #[arg(long, requires("tag"))]
        ignore_unauthorized: bool,
        /// Don't slugify the value from --tag
        #[arg(long, requires("tag"))]
        no_slug: bool,
        /// only check --tag TAG with the local docker registry
        #[arg(long, requires("tag"))]
        offline: bool,
        /// outputs in stderr the progress of fetching the tags info, similar to --verbose
        /// but without all the other details --verbose adds
        #[arg(long, requires("tag"))]
        progress: bool,
        /// max number of threads used to fetch images info
        #[arg(long, value_name = "NUM", default_value_t = 8, value_parser = positive_less_than_32, requires("tag"))]
        threads: u8,
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
    Envs {
        #[arg(value_parser = string_no_empty)]
        service: String,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, strum_macros::Display)]
pub enum Formats {
    Full,
    Oneline,
}
