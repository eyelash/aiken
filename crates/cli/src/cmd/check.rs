use std::path::PathBuf;

#[derive(clap::Args)]
/// Type-check an Aiken project
pub struct Args {
    /// Path to project
    #[clap(short, long)]
    directory: Option<PathBuf>,
}

pub fn exec(Args { directory }: Args) -> miette::Result<()> {
    crate::with_project(directory, |p| p.check())
}
