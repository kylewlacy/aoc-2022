use std::{
    collections::HashSet,
    fmt::Display,
    io::BufRead,
    ops::{Add, AddAssign, Sub},
    str::FromStr,
};

fn main() -> color_eyre::Result<()> {
    let stdin = std::io::stdin().lock();

    let mut rope = Rope::new();

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

    println!("{}", rope.tail_positions.len());

    Ok(())
}

struct Rope {
    head_position: Position,
    tail_position: Position,
    tail_positions: HashSet<Position>,
}

impl Rope {
    fn new() -> Self {
        let initial_posiiton = Position { x: 0, y: 0 };
        Self {
            head_position: initial_posiiton,
            tail_position: initial_posiiton,
            tail_positions: HashSet::from([initial_posiiton]),
        }
    }

    fn move_head(&mut self, direction: Direction) {
        self.head_position += direction.vector();

        self.adjust_tail_position();

        self.tail_positions.insert(self.tail_position);
    }

    fn adjust_tail_position(&mut self) {
        if self.head_position.is_touching(self.tail_position) {
            return;
        }

        let adjustment = (self.head_position - self.tail_position).normalize();

        self.tail_position += adjustment;
    }
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
