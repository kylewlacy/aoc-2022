use std::{cmp::Reverse, io::BufRead, str::FromStr};

use joinery::JoinableIterator;
use regex::Regex;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().without_time())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let stdin = std::io::stdin().lock();
    let mut lines = stdin.lines();

    let mut monkeys = vec![];

    while let Some(header_line) = lines.next() {
        let header_line = header_line?;
        if header_line.is_empty() {
            continue;
        }

        let header_captures = HEADER_REGEX
            .captures(&header_line)
            .ok_or_else(|| eyre::eyre!("invalid header: {header_line}"))?;
        let monkey_index = header_captures.get(1).unwrap();
        let monkey_index: usize = monkey_index.as_str().parse().unwrap();
        eyre::ensure!(
            monkey_index == monkeys.len(),
            "expected index {}, got {monkey_index}",
            monkeys.len()
        );

        let items_line = lines
            .next()
            .ok_or_else(|| eyre::eyre!("no items for monkey {monkey_index}"))?;
        let items_line = items_line?;
        let items_capture = ITEMS_REGEX
            .captures(&items_line)
            .ok_or_else(|| eyre::eyre!("invalid items for monkey {monkey_index}: {items_line}"))?;
        let items = items_capture.get(1).unwrap();
        let items = items.as_str().split(", ").map(|item_worry| {
            let worry = item_worry.parse()?;
            eyre::Result::Ok(Item { worry })
        });
        let items = items.collect::<eyre::Result<Vec<_>>>()?;

        let operation_line = lines
            .next()
            .ok_or_else(|| eyre::eyre!("no operation for monkey {monkey_index}"))?;
        let operation_line = operation_line?;
        let operation_capture = OPERATION_REGEX.captures(&operation_line).ok_or_else(|| {
            eyre::eyre!("invalid operation for monkey {monkey_index}: {operation_line}")
        })?;
        let operation = operation_capture.get(1).unwrap();
        let operation: Operation = operation.as_str().parse()?;

        let test_line = lines
            .next()
            .ok_or_else(|| eyre::eyre!("no test for monkey {monkey_index}"))?;
        let test_line = test_line?;
        let test_capture = TEST_REGEX
            .captures(&test_line)
            .ok_or_else(|| eyre::eyre!("invalid test for monkey {monkey_index}: {test_line}"))?;
        let test = test_capture.get(1).unwrap();
        let test: Test = test.as_str().parse()?;

        let condition_1_line = lines
            .next()
            .ok_or_else(|| eyre::eyre!("condition 1 not found for monkey {monkey_index}"))?;
        let condition_1_line = condition_1_line?;
        let condition_1_capture = CONDITION_REGEX.captures(&condition_1_line).ok_or_else(|| {
            eyre::eyre!("condition 1 invalid for monkey {monkey_index}: {condition_1_line}")
        })?;
        let condition_1_when = condition_1_capture.get(1).unwrap().as_str();
        let condition_1_action: Action = condition_1_capture.get(2).unwrap().as_str().parse()?;

        let condition_2_line = lines
            .next()
            .ok_or_else(|| eyre::eyre!("condition 2 not found for monkey {monkey_index}"))?;
        let condition_2_line = condition_2_line?;
        let condition_2_capture = CONDITION_REGEX.captures(&condition_2_line).ok_or_else(|| {
            eyre::eyre!("condition 2 invalid for monkey {monkey_index}: {condition_1_line}")
        })?;
        let condition_2_when = condition_2_capture.get(1).unwrap().as_str();
        let condition_2_action: Action = condition_2_capture.get(2).unwrap().as_str().parse()?;

        let (if_true, if_false) = match (condition_1_when, condition_2_when) {
            ("true", "false") => (condition_1_action, condition_2_action),
            _ => {
                eyre::bail!("invalid combination of conditions for monkey {monkey_index}");
            }
        };

        let condition = Condition {
            test,
            if_true,
            if_false,
        };

        let monkey = Monkey {
            inspections: 0,
            items,
            operation,
            condition,
        };

        monkeys.push(monkey);
    }

    let monkey_business = play_keep_away(monkeys);

    println!("{monkey_business}");

    Ok(())
}

lazy_static::lazy_static! {
    static ref HEADER_REGEX: Regex = Regex::new(r#"^Monkey (\d+):$"#).unwrap();
    static ref ITEMS_REGEX: Regex = Regex::new(r##"^\s+Starting items: ([\d, ]+)$"##).unwrap();
    static ref OPERATION_REGEX: Regex = Regex::new(r##"^\s+Operation: new = (.+)$"##).unwrap();
    static ref TEST_REGEX: Regex = Regex::new(r##"^\s+Test: (divisible by \d+)$"##).unwrap();
    static ref CONDITION_REGEX: Regex = Regex::new(r##"\s+If (true|false): (throw to monkey \d+)$"##).unwrap();
}

fn play_keep_away(mut monkeys: Vec<Monkey>) -> usize {
    for round in 1..=20 {
        for i in 0..monkeys.len() {
            tracing::trace!("Monkey {i}:");
            let outcomes = monkeys[i].play_turn();
            for outcome in outcomes {
                match outcome {
                    Outcome::ThrowToMonkey { item, target } => {
                        monkeys[target].items.push(item);
                    }
                }
            }
        }

        tracing::debug!(
            "After round {round}, the monkeys are holding items with these worry levels:"
        );
        for (i, monkey) in monkeys.iter().enumerate() {
            tracing::debug!(
                "Monkey {i}: {}",
                monkey
                    .items
                    .iter()
                    .map(|item| lazy_format::lazy_format!("{}", item.worry))
                    .join_with(", ")
            );
        }
        tracing::debug!("");
    }

    monkeys.sort_by_key(|monkey| Reverse(monkey.inspections));

    let monkey_business = monkeys
        .iter()
        .take(2)
        .map(|monkey| monkey.inspections)
        .product();
    monkey_business
}

#[derive(Debug)]
struct Monkey {
    inspections: usize,
    items: Vec<Item>,
    operation: Operation,
    condition: Condition,
}

impl Monkey {
    fn play_turn(&mut self) -> Vec<Outcome> {
        let mut outcomes = vec![];

        for mut item in self.items.drain(..) {
            tracing::trace!(
                "  Monkey inspect an item with a worry level of {}",
                item.worry
            );

            // Inspect the item
            item.worry = self.operation.apply(item.worry);

            tracing::trace!("    Worry level becomes {}", item.worry);

            // Relief from the item not being damaged
            item.worry /= 3;

            tracing::trace!(
                "    Monkey gets bored with item. Worry level is divided by 3 to {}",
                item.worry
            );

            // Result of the inspection
            let action = self.condition.action(item.worry);
            let outcome = match *action {
                Action::ThrowToMonkey(target) => {
                    tracing::trace!(
                        "    Item with worry level {} is thrown to monkey {target}",
                        item.worry
                    );
                    Outcome::ThrowToMonkey { item, target }
                }
            };
            outcomes.push(outcome);

            // Count the inspection
            self.inspections += 1;
        }

        outcomes
    }
}

#[derive(Debug)]
struct Item {
    worry: i64,
}

#[derive(Debug, Clone, Copy)]
enum Operation {
    Add(Operand, Operand),
    Multiply(Operand, Operand),
}

impl Operation {
    fn apply(&self, old: i64) -> i64 {
        match self {
            Operation::Add(op1, op2) => op1.apply(old) + op2.apply(old),
            Operation::Multiply(op1, op2) => op1.apply(old) * op2.apply(old),
        }
    }
}

impl FromStr for Operation {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.split_whitespace();
        let operand_1 = tokens
            .next()
            .ok_or_else(|| eyre::eyre!("expected more tokens"))?;
        let operand_1: Operand = operand_1.parse()?;

        let operator = tokens
            .next()
            .ok_or_else(|| eyre::eyre!("expected more tokens"))?;

        let operand_2 = tokens
            .next()
            .ok_or_else(|| eyre::eyre!("expected more tokens"))?;
        let operand_2: Operand = operand_2.parse()?;

        if let Some(_) = tokens.next() {
            eyre::bail!("unexpected token in operation: {s}");
        }

        match operator {
            "+" => Ok(Self::Add(operand_1, operand_2)),
            "*" => Ok(Self::Multiply(operand_1, operand_2)),
            other => eyre::bail!("unknown operator {other:?} in operation: {s}"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Operand {
    Value(i64),
    Old,
}

impl Operand {
    fn apply(&self, old: i64) -> i64 {
        match self {
            Operand::Value(value) => *value,
            Operand::Old => old,
        }
    }
}

impl FromStr for Operand {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "old" => Ok(Self::Old),
            value => {
                let value = value.parse()?;
                Ok(Self::Value(value))
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Condition {
    test: Test,
    if_true: Action,
    if_false: Action,
}

impl Condition {
    fn action(&self, value: i64) -> &Action {
        if self.test.passes(value) {
            &self.if_true
        } else {
            &self.if_false
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Test {
    DivisibleBy(i64),
}

impl Test {
    fn passes(&self, value: i64) -> bool {
        match self {
            Test::DivisibleBy(divisor) => value % divisor == 0,
        }
    }
}

impl FromStr for Test {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once("divisible by ") {
            Some(("", divisor)) => {
                let divisor = divisor.parse()?;
                Ok(Self::DivisibleBy(divisor))
            }
            _ => {
                eyre::bail!("invalid condition: {s}");
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Action {
    ThrowToMonkey(usize),
}

impl FromStr for Action {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once("throw to monkey ") {
            Some(("", to_monkey_index)) => {
                let to_monkey_index = to_monkey_index.parse()?;
                Ok(Self::ThrowToMonkey(to_monkey_index))
            }
            _ => {
                eyre::bail!("invalid action: {s}");
            }
        }
    }
}

enum Outcome {
    ThrowToMonkey { item: Item, target: usize },
}
