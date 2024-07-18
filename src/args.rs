/// Types to parse the command line arguments with the clap crate.
use crate::{header, positive_less_than_32, string_no_empty, string_script, Verbosity};
use clap::{Parser, Subcommand, ValueEnum};
use std::cmp::Ord;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long = "file", value_name = "FILENAME")]
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
    #[arg(long, conflicts_with = "no_docker")]
    pub no_consistency: bool,
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
        /// output image attributes in services with a tag passed instead of the one set in the file
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

    /// Download a file from an HTTP URL, if the resource doesn't exist, fallback
    /// to another URL generated editing the URL given with a script provided in the
    /// form of "text-to-replace:replacer".
    Get {
        /// The URL where the file is located
        #[arg(value_parser = string_no_empty)]
        url: String,
        /// if request to URL responds back with HTTP 404, create a second URL
        /// replacing any occurrence of the left part of the script with the right
        /// part. Each part of the script has to be separated with the symbol `:`.
        /// E.g. `pose get https://server.com/repo/feature-a/compose.yml feature-a:master`
        /// will try first download the resource from https://server.com/repo/feature-a/compose.yml,
        /// if not found, will try at https://server.com/repo/master/compose.yml
        #[arg(value_parser = string_script)]
        script: Option<(String, String)>,
        /// Save to file (default use the same filename set in the url)
        #[arg(short, long, value_name = "FILE")]
        output: Option<String>,
        /// Maximum time in seconds that you allow pose's connection to take.
        /// This only limits the connection phase, so if pose connects within the
        /// given period it will continue, if not it will exit with error.
        #[arg(long, value_name = "SECONDS", default_value_t = 30)]
        timeout_connect: u16,
        /// Maximum time in seconds that you allow the whole operation to take.
        #[arg(short, long, value_name = "SECONDS", default_value_t = 300)]
        max_time: u16,
        /// HTTP header to include in the request
        #[arg(short = 'H', long = "header", value_name = "HEADER", value_parser = header)]
        headers: Vec<(String, String)>,
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
