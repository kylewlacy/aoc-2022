use std::{fmt::Display, io::BufRead, str::FromStr};

use joinery::JoinableIterator;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{complete, map, map_res},
    error::VerboseError,
    multi::separated_list0,
    sequence::delimited,
    IResult,
};

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let stdin = std::io::stdin().lock();
    let mut lines = stdin.lines();
    let mut index = 1;
    let mut sum_correctly_ordered_indices = 0;
    while let Some(line_left) = lines.next() {
        let line_left = line_left?;

        let line_right = lines.next().ok_or_else(|| eyre::eyre!("no right line"))??;

        match lines.next() {
            Some(Ok(blank)) if blank.is_empty() => {}
            None => {}
            Some(Ok(non_blank)) => {
                eyre::bail!("unexpected line after right packet: {non_blank:?}");
            }
            Some(Err(err)) => {
                eyre::bail!(err);
            }
        }

        let left_packet: Packet = line_left.parse()?;
        let right_packet: Packet = line_right.parse()?;

        if left_packet < right_packet {
            sum_correctly_ordered_indices += index;
        }

        index += 1;
    }

    println!("{sum_correctly_ordered_indices}");

    Ok(())
}

#[derive(Debug, Clone)]
enum Packet {
    Number(u32),
    List(Vec<Packet>),
}

impl Ord for Packet {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let result = match (self, other) {
            (Packet::Number(left), Packet::Number(right)) => left.cmp(right),
            (Packet::List(left), Packet::List(right)) => {
                let left = left.iter().map(Some).chain(std::iter::repeat(None));
                let right = right.iter().map(Some).chain(std::iter::repeat(None));
                for (left, right) in left.zip(right) {
                    match (left, right) {
                        (Some(left), Some(right)) => match left.cmp(right) {
                            std::cmp::Ordering::Equal => {
                                // Values are equal, so keep iterating
                            }
                            cmp => return cmp,
                        },
                        (None, None) => return std::cmp::Ordering::Equal,
                        (None, Some(_)) => return std::cmp::Ordering::Less,
                        (Some(_), None) => return std::cmp::Ordering::Greater,
                    }
                }

                // The iterator above is infinite
                unreachable!();
            }
            (Packet::Number(left), right @ Packet::List(_)) => {
                Packet::List(vec![Packet::Number(*left)]).cmp(right)
            }
            (left @ Packet::List(_), Packet::Number(right)) => {
                left.cmp(&Packet::List(vec![Packet::Number(*right)]))
            }
        };

        result
    }
}

impl PartialOrd for Packet {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Packet {}

impl PartialEq for Packet {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl Display for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Packet::Number(value) => write!(f, "{}", value),
            Packet::List(values) => {
                write!(f, "[{}]", values.iter().join_with(", "))
            }
        }
    }
}

impl FromStr for Packet {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parser = complete(parse_packet);
        let (_, value) = parser(s).map_err(|err| eyre::eyre!("parse error: {err}"))?;

        Ok(value)
    }
}

fn parse_packet<'a>(i: &'a str) -> IResult<&'a str, Packet, VerboseError<&'a str>> {
    let mut parser = alt((
        map(parse_packet_number, Packet::Number),
        map(parse_packet_list, Packet::List),
    ));
    parser(i)
}

fn parse_packet_number<'a>(i: &'a str) -> IResult<&'a str, u32, VerboseError<&'a str>> {
    let mut parser = map_res(digit1, |s: &str| s.parse());
    parser(i)
}

fn parse_packet_list<'a>(i: &'a str) -> IResult<&'a str, Vec<Packet>, VerboseError<&'a str>> {
    let mut parser = delimited(tag("["), separated_list0(tag(","), parse_packet), tag("]"));
    parser(i)
}
