#[derive(Debug, Default, Clone)]
pub enum Verbosity {
    Quiet,
    #[default]
    Info,
    Verbose,
}
