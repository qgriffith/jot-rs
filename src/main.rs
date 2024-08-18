use clap::error::ErrorKind;
use clap::{CommandFactory, Parser, Subcommand};
use directories::UserDirs;
use miette::{IntoDiagnostic, WrapErr};
use std::path::PathBuf;

/// A ClI for jotting down notes

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    #[clap(short = 'p', long, env)]
    jot_path: Option<PathBuf>,

    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// jot something down
    ///
    /// This command will open your $EDITOR, wait for you
    /// to write something, and then save the file to your
    /// jot
    Write {
        /// Optionally set a title for what you are going to write about
        #[clap(short, long)]
        title: Option<String>,
    },
}

/// get the user's jot directory, which by default
/// is placed in their home dir
fn get_default_jot_dir() -> Option<PathBuf> {
    UserDirs::new().map(|dirs| dirs.home_dir().join("jot"))
}

fn main() -> miette::Result<()> {
    let args = Args::parse();

    let Some(jot_path) = args.jot_path.or_else(get_default_jot_dir) else {
        let mut cmd = Args::command();
        cmd.error(
            ErrorKind::ValueValidation,
            "jot directory not provided and home directory unavailable for default jot directory"
                .to_string(),
        )
        .exit()
    };
    if !jot_path.exists() {
        let mut cmd = Args::command();
        cmd.error(
            ErrorKind::ValueValidation,
            format!(
                "jot directory `{}` doesn't exist, or is inaccessible",
                jot_path.display()
            ),
        )
        .exit()
    };

    match args.cmd {
        Commands::Write { title } => jot::write(jot_path, title)
            .into_diagnostic()
            .wrap_err("jot::write"),
    }
}
