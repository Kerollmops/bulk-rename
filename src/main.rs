use std::fmt::Write as _;
use std::io::Write as _;
use std::process::Command;
use std::{env, fs};

use main_error::MainError;

fn main() -> Result<(), MainError> {
    let editor = env::var("EDITOR")?;

    let mut entries = String::new();
    for entry in fs::read_dir(".")? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            if let Some(filename) = entry.file_name().to_str() {
                writeln!(&mut entries, "{}", filename)?;
            }
        }
    }

    let mut file = tempfile::tempfile()?;
    file.write_all(entries.as_bytes())?;

    let output = Command::new("sh")
        .arg("-c")
        .arg(editor)
        .stdin(file)
        .output()?;

    let new_entries = String::from_utf8(output.stdout)?;

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
