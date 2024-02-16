#[derive(Clone, Debug, Default)]
pub enum Verbosity {
    Quiet,
    #[default]
    Info,
    Verbose,
}
