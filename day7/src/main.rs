use std::{collections::HashMap, io::BufRead};

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();
    let mut lines = stdin.lines().peekable();

    let mut filesystem = FilesystemEntry::dir();
    let mut current_directory = Path::root();

    while let Some(line) = lines.next() {
        let line = line?;
        let prompt = line.strip_prefix("$ ").context("could not parse command")?;
        let mut prompt = prompt.split_whitespace();
        let command = prompt.next().context("no command entered")?;

        let command = match command {
            "cd" => {
                let arg = prompt.next().context("cd: expected arg")?;
                Command::Cd(arg)
            }
            "ls" => Command::Ls,
            command => anyhow::bail!("unknown command: {command}"),
        };

        match command {
            Command::Cd("/") => current_directory = Path::root(),
            Command::Cd("..") => current_directory.up(),
            Command::Cd(subpath) => current_directory.enter(subpath.to_string()),
            Command::Ls => loop {
                let line = lines.next_if(|line| match line {
                    Ok(line) if line.starts_with("$ ") => false,
                    Ok(_) => true,
                    Err(_) => false,
                });

                let line = match line {
                    Some(Ok(line)) => line,
                    None => break,
                    Some(Err(_)) => unreachable!(), // Filtered out in next_if
                };

                let mut file_info = line.split_whitespace();
                let file_details = file_info
                    .next()
                    .context("failed to parse info field of ls command")?;
                let filename = file_info
                    .next()
                    .context("failed to parse filename field of ls command")?;

                if let Some(_) = prompt.next() {
                    anyhow::bail!("unexpected field in ls line: {line}");
                }

                let filesystem_entry = match file_details {
                    "dir" => FilesystemEntry::dir(),
                    size => {
                        let size = size.parse().context("invalid filesize")?;
                        FilesystemEntry::File(File { size })
                    }
                };

                filesystem.insert(&current_directory, filename.to_owned(), filesystem_entry)?;
            },
        }

        if let Some(arg) = prompt.next() {
            anyhow::bail!("unexpected argument for command {command:?}: {arg}");
        }
    }

    println!("{filesystem:#?}");

    Ok(())
}

#[derive(Debug)]
enum Command<'a> {
    Cd(&'a str),
    Ls,
}

#[derive(Debug, Clone)]
struct Path {
    components: Vec<String>,
}

impl Path {
    fn root() -> Self {
        Self { components: vec![] }
    }

    fn up(&mut self) {
        self.components.pop();
    }

    fn enter(&mut self, filename: String) {
        self.components.push(filename);
    }
}

#[derive(Debug)]
enum FilesystemEntry {
    Directory(Directory),
    File(File),
}

impl FilesystemEntry {
    fn dir() -> Self {
        Self::Directory(Directory::empty())
    }

    fn insert(
        &mut self,
        current_directory: &Path,
        filename: String,
        entry: FilesystemEntry,
    ) -> anyhow::Result<()> {
        let mut dir = match self {
            Self::Directory(dir) => dir,
            Self::File(_) => anyhow::bail!("not a directory"),
        };
        for path_component in &current_directory.components {
            dir = match dir.entries.get_mut(path_component) {
                Some(Self::Directory(dir)) => dir,
                Some(Self::File(_)) => anyhow::bail!("not a directory: {path_component}"),
                None => anyhow::bail!("file not found: {path_component}"),
            };
        }

        dir.entries.insert(filename, entry);

        Ok(())
    }
}

#[derive(Debug)]
struct Directory {
    entries: HashMap<String, FilesystemEntry>,
}

impl Directory {
    fn empty() -> Self {
        Directory {
            entries: HashMap::new(),
        }
    }
}

#[derive(Debug)]
struct File {
    size: u64,
}
