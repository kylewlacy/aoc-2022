use std::{collections::HashMap, io::BufRead};

use anyhow::Context;
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[clap(long)]
    total_disk_space: u64,
    #[clap(long)]
    target_unused_space: u64,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

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

    let current_unused_space = args
        .total_disk_space
        .checked_sub(filesystem.size())
        .context("filesystem is using more than total disk space")?;
    let required_to_delete = args
        .target_unused_space
        .checked_sub(current_unused_space)
        .context("already have enough disk space")?;
    let mut directory_sizes: Vec<_> = filesystem
        .entries()
        .filter_map(|entry| match entry {
            FilesystemEntry::Directory(dir) => Some(dir.total_size),
            FilesystemEntry::File(_) => None,
        })
        .collect();

    directory_sizes.sort();

    for candidate_directory_size in directory_sizes {
        if candidate_directory_size >= required_to_delete {
            // Delete this directory and exit.
            println!("{candidate_directory_size}");

            return Ok(());
        }
    }

    anyhow::bail!("could not find a big enough directory to delete");
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
        let entry_size = entry.size();

        let mut dir = match self {
            Self::Directory(dir) => dir,
            Self::File(_) => anyhow::bail!("not a directory"),
        };
        dir.total_size += entry_size;

        for path_component in &current_directory.components {
            dir = match dir.entries.get_mut(path_component) {
                Some(Self::Directory(dir)) => dir,
                Some(Self::File(_)) => anyhow::bail!("not a directory: {path_component}"),
                None => anyhow::bail!("file not found: {path_component}"),
            };
            dir.total_size += entry_size;
        }

        dir.entries.insert(filename, entry);

        Ok(())
    }

    fn size(&self) -> u64 {
        match self {
            FilesystemEntry::Directory(dir) => dir.total_size,
            FilesystemEntry::File(file) => file.size,
        }
    }

    fn entries(&self) -> impl Iterator<Item = &FilesystemEntry> {
        let mut queue: Vec<&FilesystemEntry> = vec![self];
        std::iter::from_fn(move || {
            let current = queue.pop();
            match current {
                Some(Self::Directory(dir)) => {
                    queue.extend(dir.entries.values());
                }
                _ => {}
            }

            current
        })
    }
}

#[derive(Debug)]
struct Directory {
    total_size: u64,
    entries: HashMap<String, FilesystemEntry>,
}

impl Directory {
    fn empty() -> Self {
        Directory {
            total_size: 0,
            entries: HashMap::new(),
        }
    }
}

#[derive(Debug)]
struct File {
    size: u64,
}
