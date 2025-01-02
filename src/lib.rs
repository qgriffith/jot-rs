use edit::{edit_file, Builder};
use miette::Diagnostic;
use owo_colors::OwoColorize;
use std::{fs, io, io::Write, path::PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_from_empty_string() {
        assert_eq!(title_from_content(""), None);
    }

    #[test]
    fn title_from_content_string() {
        assert_eq!(
            title_from_content("# some title"),
            Some("some title".to_string())
        );
    }

    #[test]
    fn title_from_content_no_title() {
        assert_eq!(title_from_content("# "), None);
    }
}

/// Errors that can occur during the operation of the `jot` CLI application.
///
/// This enum represents various error types, categorized into:
/// - **I/O errors** (`IoError`)
/// - **Temporary file creation errors** (`TempfileCreationError`)
/// - **Temporary file persistence errors** (`TempfileKeepError`)
///
/// The `JotVarietyError` is built using the [`thiserror`](https://docs.rs/thiserror) crate
/// for structured error composition and utilizes the [`miette`](https://docs.rs/miette) library
/// for generating user-friendly diagnostics.
#[derive(Error, Diagnostic, Debug)]
pub enum JotVarietyError {
    #[error(transparent)]
    #[diagnostic(code(jot::io_error))]
    IoError(#[from] std::io::Error),

    #[error("failed to create tempfile: {0}")]
    #[diagnostic(code(jot::tempfile_create_error))]
    TempfileCreationError(std::io::Error),

    #[error("failed to keep tempfile: {0}")]
    #[diagnostic(code(jot::tempfile_keep_error))]
    TempfileKeepError(#[from] tempfile::PersistError),
}

fn ask_for_filename() -> io::Result<String> {
    rprompt::prompt_reply(
        "Enter filename\
        > "
        .blue()
        .bold(),
    )
}

/// Confirms or modifies an initial filename by interacting with the user in the terminal.
///
/// This function prompts the user with the current title (`raw_title`) and asks whether they want to keep it
/// or provide a new title. If the user confirms the title or provides valid input for a new title, the function
/// returns the final, confirmed filename as a `String`.
///
/// # Arguments
///
/// * `raw_title` - A string slice that represents the initial or default filename suggestion.
///
/// # Returns
///
/// This function returns a `Result<String, std::io::Error>`:
///
/// * `Ok(String)` - Contains the confirmed or newly provided title.
/// * `Err(io::Error)` - Propagates any errors encountered during the prompt interaction with the user.
///
/// # Behavior
///
/// 1. The user is prompted with:
///    ```text
///    current title: {raw_title}
///    Do you want a different title? (y/N):
///    ```
///    - The current title is displayed in bold green (styled for terminal output).
///    - The default choice is `N` (indicating no change to the title).
///
/// 2. User inputs are handled as follows:
///    - `y` or `Y`: The user opts to change the title, and the function invokes [`ask_for_filename`]
///      to allow them to input a new filename.
///    - `n`, `N`, or an empty input: The user confirms the current title, and it is returned unchanged.
///    - Any other input will repeat the prompt.
///
/// 3. If the user chooses to change the title, [`ask_for_filename`] is called to prompt for a new title.
///
fn confirm_filename(raw_title: &str) -> io::Result<String> {
    loop {
        // prompt defaults to uppercase charcter in question
        // this is a convention not a req
        let result = rprompt::prompt_reply(&format!(
            "current title: {}
Do you want a different title? (y/{}): ",
            &raw_title.bold().green(),
            "N".bold()
        ))?;

        match result.as_str() {
            "y" | "Y" => break ask_for_filename(),
            "n" | "N" | "" => {
                break Ok(raw_title.to_string());
            }
            _ => {
                // ask again something has gone wrong
            }
        }
    }
}

/// Extracts the title from the given input string by searching for a Markdown-style header.
///
/// This function looks for the first line in the input string that starts with
/// the Markdown header prefix `# `. If such a line is found, and it contains
/// non-empty content, the function will return the title as a `String`.
///
/// # Arguments
///
/// * `input` - A string slice containing the content to extract a title from.
///
/// # Returns
///
/// * `Option<String>` - Returns `Some(String)` containing the title if a valid header is found,
/// or `None` if no header is found or the header is empty.
///
fn title_from_content(input: &str) -> Option<String> {
    input.lines().find_map(|line| {
        line.strip_prefix("# ")
            .and_then(|title| (!title.is_empty()).then_some(title.to_string()))
    })
}

/// Creates a new markdown file in the specified directory and allows the user to edit its contents.
///
/// This function performs the following operations:
/// 1. Creates a temporary markdown file in the specified `jot_path` directory.
/// 2. Writes a basic markdown template header, using the provided `title` (if any).
/// 3. Opens the file in the user's `$EDITOR` for editing.
/// 4. Reads the file content and determines a final filename from the title or user input.
/// 5. Saves the file with a unique name in the specified directory.
///
/// # Arguments
///
/// * `jot_path` - A [`PathBuf`](https://doc.rust-lang.org/std/path/struct.PathBuf.html) specifying the directory
///   where the new file will be created.
/// * `title` - An optional title to set the initial header of the markdown document. If not provided,
///   the template will use an empty title, and one can be inferred from the content later.
///
/// # Returns
///
/// Returns [`Result<(), std::io::Error>`](https://doc.rust-lang.org/std/result/enum.Result.html) which:
/// - On success, returns `Ok(())`.
/// - On error, propagates any kind of I/O-related errors, such as file creation or rename failures.
///
/// # Behavior
///
/// 1. The function creates a temporary file with a `.md` suffix and a random name.
///    The file is created within the `jot_path` directory.
/// 2. A markdown template header is written to the file. If a `title` is provided, it is used as the main header;
///    otherwise, an empty header is used.
/// 3. The temporary file is opened in the user's default editor by invoking `edit_file`.
/// 4. After editing, the function reads the file’s contents to determine the title:
///    - If a `title` was already provided, it uses it directly.
///    - If no title was provided, the function looks for the first markdown-style header (`#`) in the content
///      using [`title_from_content`].
/// 5. The user is then prompted to confirm or modify the resulting filename using [`confirm_filename`].
/// 6. The file is renamed and saved to the final location:
///    - Filename conflicts are automatically resolved by appending `-001`, `-002`, etc., to the new filename.
///
/// # Error Handling
///
/// This function might fail due to:
/// - File creation issues in the provided `jot_path`.
/// - Failure to open the file for editing or rename the file on save.
/// - Failures during user prompts if they involve I/O errors.
///
pub fn write(jot_path: PathBuf, title: Option<String>) -> Result<(), std::io::Error> {
    let (mut file, filepath) = Builder::new()
        .suffix(".md")
        .rand_bytes(5)
        .tempfile_in(&jot_path)?
        .keep()?;
    let template = format!("# {}", title.as_deref().unwrap_or(""));
    file.write_all(template.as_bytes())?;
    edit_file(&filepath)?;
    let contents = fs::read_to_string(&filepath)?;
    let document_title = title.or_else(|| title_from_content(&contents));

    let filename = match document_title {
        Some(raw_title) => confirm_filename(&raw_title),
        None => ask_for_filename(),
    }
    .map(|title| slug::slugify(title))?;

    for attempt in 0.. {
        let mut dest = jot_path.join(if attempt == 0 {
            filename.clone()
        } else {
            format!("{filename}{:03}", -attempt)
        });
        dest.set_extension("md");
        if dest.exists() {
            continue;
        }
        fs::rename(filepath, &dest)?;
        break;
    }

    Ok(())
}

/// Appends a message to a scratch file named `_scratch.md` in the given directory.
///
/// This function creates or appends to a file named `_scratch.md` in the specified
/// `jot_path`. The append operation makes it suitable for jotting down quick
/// notes or reminders. If the `_scratch.md` file does not exist, it will
/// automatically be created.
///
/// # Arguments
///
/// * `jot_path` - A `PathBuf` specifying the directory where the `_scratch.md`
///   file is located or will be created.
/// * `message` - The `String` content to be added as a new line in the `_scratch.md` file.
///
/// # Returns
///
/// This function returns a `Result`:
/// - `Ok(())` on success, indicating the message has been written to the file.
/// - `Err(std::io::Error)` if there is an issue creating or writing to the file.
///
/// # Errors
///
/// This function will return an error in situations such as:
/// - The `jot_path` directory is not accessible.
/// - There is an error creating or opening `_scratch.md` file.
/// - Writing the message to the file fails.
///
pub fn scratch(jot_path: PathBuf, message: String) -> Result<(), std::io::Error> {
    let mut scratch_path = jot_path.join("_scratch");
    scratch_path.set_extension("md");

    let mut scratch_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(scratch_path)?;
    write!(&mut scratch_file, "\n{}", message)?;
    Ok(())
}

/// Opens an existing markdown file in the specified `jot_path` directory by its title.
///
/// If the file exists, it will be opened using the provided `edit_file` function. If
/// the file does not exist, an error message will be displayed to the user.
///
/// # Arguments
///
/// * `jot_path` - A [`PathBuf`] representing the directory where the notes are stored.
/// * `title` - A [`String`] containing the name of the file to open (without the extension).
///
/// # Returns
///
/// Returns a [`Result`] with `Ok(())` if the operation succeeds, or an [`std::io::Error`]
/// if an error occurs.
///
/// # Behavior
///
/// 1. Joins the `jot_path` and the `title` to generate the file's path.
/// 2. Sets the file's extension to `.md`.
/// 3. If the file is found:
///     - Opens the file using the `edit_file` function.
/// 4. If the file is not found:
///     - Prints an error message to the user with the file path.
/// 5. Debug information about the file path is logged using `dbg!`.
///
pub fn open(jot_path: PathBuf, title: String) -> Result<(), std::io::Error> {
    let mut open_path = jot_path.join(title);
    open_path.set_extension("md");

    if open_path.is_file() {
        edit_file(&open_path)?;
    } else {
        println!(
            "{} - {}",
            "File not found".bold().red(),
            &open_path.display()
        );
    }
    Ok(())
}

/// Searches recursively within a given directory for files containing a specific query string.
///
/// # Arguments
///
/// * `jot_path` - A [`PathBuf`] representing the directory to search.
/// * `query` - A [`String`] representing the text to search for within the files.
///
/// # Return
///
/// Returns an [`Ok(())`] result if the function completes successfully. If an error occurs
/// during file reading or traversal, it returns an [`Err`] with the specific [`std::io::Error`].
///
/// # Behavior
///
/// - The function iterates over all files in the directory specified by `jot_path`, including
///   subdirectories.
/// - All text files are read, and their contents are checked for the presence of the `query` string.
/// - If a file contains the query, the path to that file is printed in bold green text.
/// - If no files contain the query, a "No results found" message is displayed in bold red text.
///
/// # Errors
///
/// - Returns an error if the directory cannot be read,
///   or if the contents of any file fail to be read.
///
pub fn search(jot_path: PathBuf, query: String) -> Result<(), std::io::Error> {
    let mut counter = 0;
    for entry in WalkDir::new(jot_path)
        .into_iter()
        .filter_map(|e| e.ok().and_then(|e2| e2.path().is_file().then_some(e2)))
    {
        let content = fs::read_to_string(&entry.path())?;
        if content.contains(&query) {
            println!("{}", entry.path().display().bold().green());
            counter += 1;
        }
    }
    // Counter is zero so that must mean the search didn't find anything
    if counter == 0 {
        println!("{}", "No results found".bold().red());
    }
    Ok(())
}

/// Lists all files in the specified directory and prints them to the console.
///
/// This function traverses the provided `jot_path` directory
/// (and its subdirectories) to find all files, excluding directories.
/// It then prints out the file paths in a bold green font for easy readability.
///
/// # Arguments
///
/// * `jot_path` - A [`PathBuf`] representing the directory where the notes are stored.
///
/// # Returns
///
/// Returns a [`Result`] with `Ok(())` if the operation completes successfully,
/// or an [`std::io::Error`] if an error occurs during file traversal.
///
/// # Behavior
///
/// 1. Recursively iterates through the `jot_path` directory using [`WalkDir`].
/// 2. Filters out directories, keeping only files.
/// 3. For each file:
///     - Prints the file path to the console, formatted in bold green text using [`owo-colors`].
///

pub fn list(jot_path: PathBuf) -> Result<(), std::io::Error> {
    for entry in WalkDir::new(jot_path)
        .into_iter()
        .filter_map(|e| e.ok().and_then(|e2| e2.path().is_file().then_some(e2)))
    {
        println!("{}", entry.path().display().bold().green());
    }
    Ok(())
}
