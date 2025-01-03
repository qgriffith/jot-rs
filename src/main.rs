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
    /// jot something new down
    ///
    /// This command will open your $EDITOR, wait for you
    /// to write something, and then save the file to your
    /// jot. This is to create a new file and save it
    New {
        /// Optionally set a title for what you are going to write about
        #[clap(short, long)]
        title: Option<String>,
    },
    /// Running jot doc that appends text to the same file.
    Open {
        /// Open an existing file to write
        #[clap(short, long)]
        title: String,
    },
    /// Used as a scratch pad
    ///
    /// To get thoughts into quickly. Perfect to use as a reminder for something
    /// to work out later. File name is _scratch.md
    Scratch {
        #[clap(short, long)]
        message: String,
    },
    /// Search documents for a string of text
    Search {
        #[clap(short, long)]
        query: String,
    },
    /// List all files in your jot dir
    List {},
}

/// get the user's jot directory, which by default
/// is placed in their home dir
fn get_default_jot_dir() -> Option<PathBuf> {
    UserDirs::new().map(|dirs| dirs.home_dir().join("jot"))
}

/// Main function that serves as the entry point for the jot CLI application.
///
/// The `main` function is responsible for parsing user input, validating input parameters,
/// and dispatching commands to the appropriate handler functions such as `jot::write`,
/// `jot::open`, `jot::scratch`, `jot::search`, and `jot::list`.
///
/// # Behavior
///
/// 1. Parses command-line arguments using [`clap`].
/// 2. Validates the `jot_path` either provided by the user or retrieved via [`get_default_jot_dir`].
///     - If neither is available, an error is raised and the program exits.
///     - If the `jot_path` does not exist, an error is shown and the program exits.
/// 3. Executes the corresponding behavior for the subcommand provided by the user.
///
/// Supported subcommands:
/// - `New`: Creates a new jot (note) using the optional title.
/// - `Open`: Edits an existing note specified by title.
/// - `Scratch`: Writes a quick note to the `_scratch.md` file.
/// - `Search`: Searches notes in the specified jot directory.
/// - `List`: Prints all files in the jot directory.
///
/// # Arguments
///
/// The main function does not take direct arguments but instead parses
/// command-line input via `Args::parse`, which is defined as:
/// ```
/// #[derive(Parser, Debug)]
/// struct Args {
///     #[clap(short = 'p', long, env)]
///     jot_path: Option<PathBuf>,
///
///     #[command(subcommand)]
///     cmd: Commands,
/// }
/// ```
///
/// ## Example Commands:
/// ```bash
/// jot new --title "My First Note"
/// jot open --title "My First Note"
/// jot scratch --message "Quick thought for today"
/// jot search --query "today"
/// jot list
/// ```
///
/// # Errors
///
/// This function returns `Err(miette::Error)` on failure. Strong input validation is performed,
/// and user-friendly error messages are displayed for common issues such as:
/// - Missing `jot_path` and no default directory found.
/// - Inaccessible or non-existent `jot_path`.
/// - Errors returned from subcommand functions like `jot::write` or `jot::search`.
///
/// # Returns
///
/// Returns a [`miette::Result<()>`] indicating success or failure.
///
/// # Examples
///
/// ### Run the application with specific commands
/// ```bash
/// # List notes in the default jot directory
/// jot list
///
/// # Create a new jot with a title:
/// jot new --title "Meeting Notes"
///
/// # Add a quick message to `_scratch.md`
/// jot scratch --message "Remember to follow up tomorrow"
///
/// # Search for a term across all notes in the directory
/// jot search --query "reminder"
/// ```
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
        Commands::New { title } => jot::write(jot_path, title)
            .into_diagnostic()
            .wrap_err("jot::write"),
        Commands::Open { title } => jot::open(jot_path, title)
            .into_diagnostic()
            .wrap_err("jot::open"),
        Commands::Scratch { message } => jot::scratch(jot_path, message)
            .into_diagnostic()
            .wrap_err("jot::scratch"),
        Commands::Search { query } => jot::search(jot_path, query)
            .into_diagnostic()
            .wrap_err("jot::search"),
        Commands::List {} => jot::list(jot_path).into_diagnostic().wrap_err("jot::list"),
    }
}
