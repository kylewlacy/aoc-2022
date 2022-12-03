use std::{collections::BTreeSet, io::BufRead};

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();

    let mut common_items: Vec<char> = vec![];
    for line in stdin.lines() {
        let line = line?;
        let compartment_size = line.len() / 2;
        let (first_compartment, second_compartment) = line.split_at(compartment_size);
        let first_compartment: BTreeSet<char> = first_compartment.chars().collect();
        let second_compartment: BTreeSet<char> = second_compartment.chars().collect();

        common_items.extend(first_compartment.intersection(&second_compartment));
    }

    let total_priority: u64 = common_items
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
