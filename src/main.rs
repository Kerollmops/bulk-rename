use std::fmt::Write as _;
use std::io::Write as _;
use std::process::{Command, Stdio};
use std::ffi::OsString;
use std::{env, fs};

use main_error::MainError;

const DEFAULT_EDITOR: &str = "/usr/bin/nano";

fn main() -> Result<(), MainError> {
    let mut editor = match env::var("EDITOR") {
        Err(_) => OsString::from(DEFAULT_EDITOR),
        Ok(editor) if editor.is_empty() => OsString::from(DEFAULT_EDITOR),
        Ok(editor) => editor.into(),
    };

    let mut entries = String::new();
    for entry in fs::read_dir(".")? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            if let Some(filename) = entry.file_name().to_str() {
                writeln!(&mut entries, "{}", filename)?;
            }
        }
    }

    let mut file = tempfile::NamedTempFile::new()?;
    file.write_all(entries.as_bytes())?;

    editor.push(" ");
    editor.push(file.path());

    let output = Command::new("sh")
        .arg("-ic")
        .arg(editor)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()?;

    if !output.status.success() {
        return Err(MainError::from("Process didn't exit successfully, aborting"));
    }

    let new_entries = fs::read_to_string(file)?;

    if entries.lines().count() != new_entries.lines().count() {
        return Err(MainError::from("The number of entries must be identical"));
    }

    for (old, new) in entries.lines().zip(new_entries.lines()) {
        if old != new {
            match fs::rename(old, new) {
                Ok(_) => eprintln!("{} -> {}", old, new),
                Err(e) => eprintln!("{}: {}", old, e),
            }
        }
    }

    Ok(())
}
