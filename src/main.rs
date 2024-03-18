use std::path::PathBuf;

use anyhow::Result;
use clap::ArgMatches;

// this will find all files ending in .md in current directory and down the fs tree.
fn find_note_files(root: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = vec![];
    let dir_result = std::fs::read_dir(root).unwrap();

    for res in dir_result {
        let res = res.unwrap();
        let file_name = res.file_name();
        let file_name = file_name.to_str().unwrap();
        if res.file_type()?.is_dir() {
            let dir_path = file_name;
            let mut entries = find_note_files(&root.join(&dir_path)).unwrap();
            files.append(&mut entries);
        } else if res.file_type().unwrap().is_file()
            && !file_name.contains(".git")
            && file_name.contains(".md")
        {
            files.push(res.path());
        }
    }

    Ok(files)
}

fn list(_: &ArgMatches) -> Result<()> {
    let root = std::env::current_dir()?;
    let notes = find_note_files(&root)?;

    for note in notes {
        println!("{}", note.strip_prefix(&root).unwrap().to_str().unwrap())
    }

    Ok(())
}

fn edit(_: &ArgMatches) -> Result<()> {
    Ok(())
}

fn main() {
    let args = clap::Command::new("notemanager")
        .about("notemanager program.")
        .version("0.0.1")
        .subcommand(clap::Command::new("list"))
        .get_matches();

    match args.subcommand() {
        Some(("list", args)) => list(args).unwrap(),
        Some(("edit", args)) => list(args).unwrap(),
        _ => todo!(),
    }
}
