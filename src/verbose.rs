#[derive(Debug, Default)]
pub enum Verbosity {
    Quiet,
    #[default]
    Info,
    Verbose,
}
