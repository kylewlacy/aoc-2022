use clap::Parser;
use std::io::BufRead;

#[derive(Debug, Default)]
struct Elves {
    top_slots: usize,
    top_elves: Vec<u64>,
    current_elf: u64,
}

impl Elves {
    fn new(top_slots: usize) -> Self {
        Elves {
            top_slots,
            top_elves: Vec::with_capacity(top_slots + 1),
            current_elf: 0,
        }
    }

    fn add_current(&mut self, calories: u64) {
        self.current_elf += calories;
    }

    fn end_current(&mut self) -> &[u64] {
        let current = std::mem::replace(&mut self.current_elf, 0);
        self.top_elves.push(current);
        self.top_elves.sort_by_key(|&elf| std::cmp::Reverse(elf));
        self.top_elves.truncate(self.top_slots);

        &self.top_elves
    }
}

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
   top_slots: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let stdin = std::io::stdin().lock();

    let mut elves = Elves::new(args.top_slots);
    for line in stdin.lines() {
        let line = line?;

        if line.is_empty() {
            elves.end_current();
        } else {
            let calories: u64 = line.parse()?;
            elves.add_current(calories);
        }
    }

    let top_elves = elves.end_current();

    let top_sum: u64 = top_elves.iter().sum();
    println!("{}", top_sum);

    Ok(())
}
