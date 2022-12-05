#![feature(byte_slice_trim_ascii)]

use std::{
    collections::{BTreeMap, VecDeque},
    io::BufRead,
};

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();
    let mut lines = stdin.lines();

    let mut columns: BTreeMap<usize, VecDeque<char>> = BTreeMap::new();
    for line in &mut lines {
        let line = line?;

        if line.trim_start().starts_with('[') {
            // Parse a row of shipping containers
            for (index, container) in line.as_bytes().chunks(4).enumerate() {
                let name = match container.trim_ascii() {
                    [b'[', name, b']'] => Some(name.into()),
                    [] => None,
                    _ => {
                        anyhow::bail!(
                            "could not parse container: {:?}",
                            String::from_utf8_lossy(container)
                        );
                    }
                };

                if let Some(&name) = name {
                    let column = columns.entry(index).or_default();
                    column.push_front(name.into());
                }
            }
        } else {
            // This is the last line with shipping container indices.
            break;
        }
    }

    for line in lines {
        let line = line?;

        if line.trim().is_empty() {
            continue;
        }

        let (prefix, line) = line
            .split_once("move ")
            .context("failed to parse move command")?;
        anyhow::ensure!(prefix.is_empty());
        let (count, line) = line
            .split_once(" from ")
            .context("failed to parse move count")?;
        let (from_column, to_column) = line
            .split_once(" to ")
            .context("failed to parse move columns")?;
        let count: usize = count.parse()?;
        let from_column: u32 = from_column.parse()?;
        let to_column: u32 = to_column.parse()?;

        let from_index = column_index(from_column)?;
        let to_index = column_index(to_column)?;
        let mut from_column = std::mem::take(columns.entry(from_index).or_default());
        let mut to_column = std::mem::take(columns.entry(to_index).or_default());

        let popped = from_column.drain(from_column.len() - count..).rev();
        to_column.extend(popped);

        columns.insert(from_index, from_column);
        columns.insert(to_index, to_column);
    }

    let top_crates = columns
        .values()
        .filter_map(|column| column.back().map(|&name| char::from(name)))
        .collect::<String>();

    println!("{top_crates}");

    Ok(())
}

fn column_index(label: u32) -> anyhow::Result<usize> {
    let label: usize = label.try_into()?;
    Ok(label - 1)
}
