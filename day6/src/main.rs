use anyhow::Context;
use itertools::Itertools;

fn main() -> anyhow::Result<()> {
    let datastream = std::io::stdin()
        .lines()
        .next()
        .context("no input provided")?
        .context("failed to read input")?;

    let sync_index =
        datastream
            .as_bytes()
            .windows(14)
            .enumerate()
            .find_map(|(start_index, bytes)| {
                for (a, b) in bytes.iter().tuple_combinations() {
                    if a == b {
                        return None;
                    }
                }

                Some(start_index + bytes.len())
            });

    let sync_index = sync_index.context("could not sync datastream")?;

    println!("{sync_index}");

    Ok(())
}
