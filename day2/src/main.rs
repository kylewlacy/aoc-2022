use std::io::BufRead;

use anyhow::Context;
use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
   top_slots: usize,
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();

    let mut total_score = 0;
    for line in stdin.lines() {
        let line = line?;
        let mut columns = line.split_whitespace();
        let opponent_move = columns.next().context("no opponent move")?;
        let my_move = columns.next().context("no response move")?;

        let opponent_move = Move::parse_opponent_move(opponent_move)?;
        let my_move = Move::parse_my_move(my_move)?;

        total_score += score_move(opponent_move, my_move);
    }

    println!("{}", total_score);

    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum Move {
    Rock,
    Paper,
    Scissors,
}

impl Move {
    fn parse_opponent_move(s: &str) -> anyhow::Result<Self> {
        match s {
            "A" => Ok(Move::Rock),
            "B" => Ok(Move::Paper),
            "C" => Ok(Move::Scissors),
            other => anyhow::bail!("unknown opponent move: {other:?}"),
        }
    }

    fn parse_my_move(s: &str) -> anyhow::Result<Self> {
        match s {
            "X" => Ok(Move::Rock),
            "Y" => Ok(Move::Paper),
            "Z" => Ok(Move::Scissors),
            other => anyhow::bail!("unknown move: {other:?}"),
        }
    }
}

fn score_move(opponent: Move, mine: Move) -> u64 {
    let shape_score = match mine {
        Move::Rock => 1,
        Move::Paper => 2,
        Move::Scissors => 3,
    };
    let outcome = match (mine, opponent) {
        (Move::Rock, Move::Rock) => Outcome::Draw,
        (Move::Rock, Move::Paper) => Outcome::Loss,
        (Move::Rock, Move::Scissors) => Outcome::Win,
        (Move::Paper, Move::Rock) => Outcome::Win,
        (Move::Paper, Move::Paper) => Outcome::Draw,
        (Move::Paper, Move::Scissors) => Outcome::Loss,
        (Move::Scissors, Move::Rock) => Outcome::Loss,
        (Move::Scissors, Move::Paper) => Outcome::Win,
        (Move::Scissors, Move::Scissors) => Outcome::Draw,
    };
    let outcome_score = match outcome {
        Outcome::Win => 6,
        Outcome::Draw => 3,
        Outcome::Loss => 0,
    };

    shape_score + outcome_score
}

enum Outcome {
    Win,
    Loss,
    Draw,
}
