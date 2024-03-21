use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use std::{fs, path::Path};

// this will find all files ending in .md in current directory and down the fs tree.
pub fn find_note_files(root: impl AsRef<Path>) -> Result<Vec<String>> {
    std::fs::read_dir(root.as_ref())
        .with_context(|| format!("Could not read directory `{:?}`", root.as_ref()))?
        .filter_map(|maybe_entry| maybe_entry.ok())
        .map(|entry| entry.path())
        .filter(|filename| filename.is_dir() || filename.is_file())
        .filter_map(|filename| filename.to_str().map(String::from))
        .try_fold(Vec::new(), |mut file_list, filename| {
            if fs::metadata(&filename).unwrap().is_dir() {
                file_list.append(&mut find_note_files(filename)?)
            } else if filename.contains(".md") && filename.ends_with(".md") {
                file_list.push(filename)
            }
            Ok(file_list)
        })
}

pub fn list() -> Result<()> {
    let root = std::env::current_dir()?;
    let notes = find_note_files(&root)?;
    notes.iter().for_each(|note| println!("{note}"));
    Ok(())
}

pub fn sync() -> Result<()> {
    std::process::Command::new("git")
        .args(["add", "."])
        .output()?;

    std::process::Command::new("git")
        .args([
            "commit",
            "-m",
            &format!("update {}", chrono::Local::now().format("%d-%m-%y %H:%M")),
        ])
        .output()?;

    std::process::Command::new("git").args(["push"]).output()?;

    Ok(())
}

pub fn edit(pattern: String) -> Result<()> {
    let root = std::env::current_dir()?;
    let matcher = SkimMatcherV2::default();
    let matches: Vec<_> = find_note_files(&root)?
        .into_iter()
        .filter(|filename| matcher.fuzzy_match(filename, &pattern).is_some())
        .collect();
    if matches.len() == 1 {
        println!("Editing {}", matches[0]);
        edit::edit_file(&matches[0])?;
        println!("File {} updated, Syncing...", matches[0]);
        sync()
    } else if matches.len() > 1 {
        println!("{}", matches.join("\n"));
        Ok(())
    } else {
        bail!("Could not find any match for pattern {pattern:?}")
    }
}

fn main() -> Result<()> {
    let command = clap::Command::new("notemanager")
        .subcommand(clap::Command::new("list"))
        .subcommand(clap::Command::new("edit").arg(clap::Arg::new("pattern")));

    let matches = command.get_matches();
    match matches.subcommand() {
        Some(("list", _args)) => list(),
        Some(("edit", _args)) => edit(_args.get_one::<String>("pattern").unwrap().to_string()),
        _ => todo!(),
    }
}
