#![feature(generators, generator_trait)]

use std::{
    cell::Cell,
    collections::BTreeSet,
    io::BufRead,
    ops::{Generator, GeneratorState},
    pin::Pin,
    str::FromStr,
};

fn main() -> color_eyre::Result<()> {
    let stdin = std::io::stdin().lock();
    let program = stdin.lines().map(|line| {
        let line = line?;
        Instruction::from_str(&line)
    });

    let system = System::new();
    let mut run_system = system.run(program);
    let mut current_cycle = 0;

    let result = loop {
        match Pin::new(&mut run_system).resume(()) {
            GeneratorState::Yielded(()) => {
                let sprite_x = system.x.get();
                let sprite_range = (sprite_x - 1)..=(sprite_x + 1);
                let screen_x = current_cycle % 40;

                if screen_x == 0 {
                    println!();
                }

                if sprite_range.contains(&screen_x) {
                    print!("#")
                } else {
                    print!(".");
                }

                current_cycle += 1;
            }
            GeneratorState::Complete(result) => {
                break result;
            }
        }
    };
    let () = result?;

    println!();

    Ok(())
}

#[derive(Debug)]
struct System {
    x: Cell<i64>,
}

impl System {
    fn new() -> Self {
        Self { x: Cell::new(1) }
    }

    fn run(
        &self,
        mut program: impl Iterator<Item = eyre::Result<Instruction>> + 'static,
    ) -> impl Generator<(), Yield = (), Return = eyre::Result<()>> + '_ {
        move || {
            while let Some(instruction) = program.next() {
                let instruction = instruction?;
                match instruction {
                    Instruction::NoOp => {
                        yield;
                    }
                    Instruction::AddX(value) => {
                        yield;
                        yield;
                        let x = self.x.get();
                        self.x.set(x + value);
                    }
                }
            }

            Ok(())
        }
    }
}

enum Instruction {
    NoOp,
    AddX(i64),
}

impl FromStr for Instruction {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut fields = s.split_whitespace();
        let opcode = fields.next().ok_or_else(|| eyre::eyre!("empty opcode"))?;
        let instruction = match opcode {
            "noop" => Self::NoOp,
            "addx" => {
                let value = fields
                    .next()
                    .ok_or_else(|| eyre::eyre!("no arg for addx"))?;
                let value = value.parse()?;
                Self::AddX(value)
            }
            unknown => eyre::bail!("unknown opcode: {unknown:?}"),
        };

        eyre::ensure!(fields.next().is_none(), "unexpected argument");

        Ok(instruction)
    }
}
