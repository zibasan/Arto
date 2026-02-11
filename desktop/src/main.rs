use clap::Parser;
use std::path::PathBuf;

const VERSION: &str = concat!(
    env!("ARTO_BUILD_VERSION"),
    " (",
    compile_time::datetime_str!(),
    ")",
);

/// Arto — the Art of Reading Markdown
#[derive(Parser, Debug)]
#[command(
    version = VERSION,
    about,
    long_about = "Arto — the Art of Reading Markdown\n\n\
        A local app that faithfully recreates GitHub-style Markdown rendering\n\
        for a beautiful reading experience.\n\n\
        Arto runs as a single instance — if already running, paths are sent\n\
        to the existing process instead of launching a new one.",
    after_long_help = "Examples:\n\
        \x20 arto                     Launch Arto (shows welcome screen)\n\
        \x20 arto README.md           Open a specific file\n\
        \x20 arto docs/               Open a directory in the file explorer\n\
        \x20 arto file1.md file2.md   Open multiple files in tabs"
)]
struct Cli {
    /// Files or directories to open
    #[arg()]
    paths: Vec<PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    if let arto::RunResult::SentToExistingInstance = arto::run(cli.paths) {
        std::process::exit(0);
    }
}
