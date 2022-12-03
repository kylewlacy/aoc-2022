#![feature(iter_array_chunks)]

use std::{collections::BTreeSet, io::BufRead};

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();

    let mut badges: Vec<char> = vec![];
    for [a, b, c] in stdin.lines().array_chunks() {
        let [a, b, c] = [a?, b?, c?];
        let a: BTreeSet<char> = a.chars().collect();
        let b: BTreeSet<char> = b.chars().collect();
        let c: BTreeSet<char> = c.chars().collect();

        let ab: BTreeSet<char> = a.intersection(&b).copied().collect();
        let abc = ab.intersection(&c);
        badges.extend(abc);
    }

    let total_priority: u64 = badges
        .iter()
        .map(|&item| -> u64 { priority(item).into() })
        .sum();
    println!("{}", total_priority);

    Ok(())
}

fn priority(item: char) -> u8 {
    match u8::try_from(item) {
        Ok(item @ b'a'..=b'z') => item - b'a' + 1,
        Ok(item @ b'A'..=b'Z') => item - b'A' + 27,
        _ => panic!("could not compute priority for item: {item:?}"),
    }
}
