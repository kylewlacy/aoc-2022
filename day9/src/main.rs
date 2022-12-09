#![feature(array_windows)]

use std::{
    cell::Cell,
    collections::HashSet,
    fmt::Display,
    io::BufRead,
    ops::{Add, AddAssign, Sub},
    str::FromStr,
};

use joinery::JoinableIterator;

fn main() -> color_eyre::Result<()> {
    let stdin = std::io::stdin().lock();

    let mut rope = Rope::new(10);

    for line in stdin.lines() {
        let line = line?;
        let mut fields = line.split_whitespace();
        let direction: Direction = fields
            .next()
            .ok_or_else(|| eyre::eyre!("no direction field"))?
            .parse()?;
        let repeat: u64 = fields
            .next()
            .ok_or_else(|| eyre::eyre!("no repeat field"))?
            .parse()?;

        for _ in 0..repeat {
            rope.move_head(direction);
        }
    }

    // println!("{}", rope.display_rope());
    // println!();

    println!("{}", rope.last_positions.len());

    Ok(())
}

struct Rope {
    knot_positions: Vec<Cell<Position>>,
    last_positions: HashSet<Position>,
}

impl Rope {
    fn new(knots: usize) -> Self {
        let initial_posiiton = Position { x: 0, y: 0 };
        Self {
            knot_positions: vec![Cell::new(initial_posiiton); knots],
            last_positions: HashSet::from([initial_posiiton]),
        }
    }

    fn move_head(&mut self, direction: Direction) {
        if let Some(first) = self.knot_positions.first_mut() {
            let first = first.get_mut();
            *first += direction.vector();
        }

        for [head, tail] in self.knot_positions.array_windows() {
            tail.set(adjust_tail_position(head.get(), tail.get()));
        }

        if let Some(last) = self.knot_positions.last() {
            self.last_positions.insert(last.get());
        }
    }

    #[allow(unused)]
    fn display_rope(&self) -> impl Display + '_ {
        let knot_positions = self.knot_positions.iter().map(|pos| pos.get());
        let x_min = knot_positions.clone().map(|pos| pos.x).min().unwrap();
        let x_max = knot_positions.clone().map(|pos| pos.x).max().unwrap();
        let y_min = knot_positions.clone().map(|pos| pos.y).min().unwrap();
        let y_max = knot_positions.clone().map(|pos| pos.y).max().unwrap();

        let y_bounds = ((y_min - 1)..=(y_max + 1)).rev(); // Reverse to go from top to bottom

        y_bounds
            .map(move |y| {
                let x_bounds = (x_min - 1)..=(x_max + 1);
                ((x_min - 1)..=(x_max + 1))
                    .map(move |x| {
                        let pos = Position { x, y };
                        self.knot_positions
                            .iter()
                            .enumerate()
                            .find_map(|(n, knot)| {
                                if knot.get() == pos {
                                    match n.try_into().unwrap() {
                                        0 => Some('H'),
                                        n => Some(char::from_digit(n, 16).unwrap_or('-')),
                                    }
                                } else {
                                    None
                                }
                            })
                            .unwrap_or('.')
                    })
                    .join_concat()
            })
            .join_with("\n")
    }
}

fn adjust_tail_position(head: Position, tail: Position) -> Position {
    if head.is_touching(tail) {
        return tail;
    }

    let adjustment = (head - tail).normalize();

    tail + adjustment
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    const NEIGHBOR_OFFSETS: [Vector; 9] = [
        Vector { x: -1, y: -1 },
        Vector { x: -1, y: 0 },
        Vector { x: -1, y: 1 },
        Vector { x: 0, y: -1 },
        Vector { x: 0, y: 0 },
        Vector { x: 0, y: 1 },
        Vector { x: 1, y: -1 },
        Vector { x: 1, y: 0 },
        Vector { x: 1, y: 1 },
    ];

    fn is_touching(self, other: Position) -> bool {
        for offset in Self::NEIGHBOR_OFFSETS {
            if self + offset == other {
                return true;
            }
        }

        return false;
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pos({}, {})", self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Vector {
    pub x: i32,
    pub y: i32,
}

impl Vector {
    fn normalize(self) -> Self {
        let x = match self.x {
            i32::MIN..=-1 => -1,
            0 => 0,
            1..=i32::MAX => 1,
        };
        let y = match self.y {
            i32::MIN..=-1 => -1,
            0 => 0,
            1..=i32::MAX => 1,
        };

        Self { x, y }
    }
}

impl Display for Vector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vec({}, {})", self.x, self.y)
    }
}

impl Add<Vector> for Position {
    type Output = Position;

    fn add(self, rhs: Vector) -> Self::Output {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign<Vector> for Position {
    fn add_assign(&mut self, rhs: Vector) {
        *self = *self + rhs;
    }
}

impl Add<Position> for Vector {
    type Output = Position;

    fn add(self, rhs: Position) -> Self::Output {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<Position> for Position {
    type Output = Vector;

    fn sub(self, rhs: Position) -> Self::Output {
        Vector {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn vector(self) -> Vector {
        match self {
            Direction::Up => Vector { x: 0, y: 1 },
            Direction::Down => Vector { x: 0, y: -1 },
            Direction::Left => Vector { x: -1, y: 0 },
            Direction::Right => Vector { x: 1, y: 0 },
        }
    }
}

impl FromStr for Direction {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "U" => Ok(Self::Up),
            "D" => Ok(Self::Down),
            "L" => Ok(Self::Left),
            "R" => Ok(Self::Right),
            other => Err(eyre::eyre!("invalid direction: {other:?}")),
        }
    }
}
