use std::{io::BufRead, ops::RangeInclusive};

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();

    let mut partial_overlaps = 0;
    for line in stdin.lines() {
        let line = line?;
        let (first, second) = line.split_once(',').context("could not split pair")?;
        let (first_a, first_b) = first
            .split_once('-')
            .context("could not split first range")?;
        let (second_a, second_b) = second
            .split_once('-')
            .context("could not split second range")?;
        let first = first_a.parse::<u64>()?..=first_b.parse::<u64>()?;
        let second = second_a.parse::<u64>()?..=second_b.parse::<u64>()?;
        if partial_overlap(&first, &second) {
            partial_overlaps += 1;
        }
    }

    println!("{partial_overlaps}");

    Ok(())
}

fn complete_overlap(first: &RangeInclusive<u64>, second: &RangeInclusive<u64>) -> bool {
    // |--------------|
    //     |-----|
    // fs             fe
    //     ss    se
    // fs <= ss && fe >= se
    (first.contains(second.start()) && first.contains(second.end()))
        || (second.contains(first.start()) && second.contains(first.end()))
}

fn partial_overlap(first: &RangeInclusive<u64>, second: &RangeInclusive<u64>) -> bool {
    complete_overlap(first, second)
        || ((first.contains(second.start()) || first.contains(second.end()))
            && (second.contains(first.start()) || second.contains(second.end())))
}
