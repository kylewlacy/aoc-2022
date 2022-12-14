use std::{
    fmt::Display,
    ops::{Add, AddAssign, RangeInclusive, Sub},
    str::FromStr,
};

use joinery::JoinableIterator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl FromStr for Point {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x, y) = s
            .split_once(',')
            .ok_or_else(|| eyre::eyre!("invalid point: {s:?}"))?;
        let x = x.parse()?;
        let y = y.parse()?;

        Ok(Self { x, y })
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vector {
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

impl Add<Vector> for Point {
    type Output = Point;

    fn add(self, rhs: Vector) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add<Point> for Vector {
    type Output = Point;

    fn add(self, rhs: Point) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign<Vector> for Point {
    fn add_assign(&mut self, rhs: Vector) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Point {
    type Output = Vector;

    fn sub(self, rhs: Self) -> Vector {
        Vector {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub start: Point,
    pub end: Point,
}

impl Line {
    pub fn points(&self) -> impl Iterator<Item = Point> + '_ {
        let mut current = self.start;
        let vector = (self.end - self.start).normalize();

        let mut running = true;
        std::iter::from_fn(move || {
            if current != self.end {
                let point = current;
                current += vector;
                Some(point)
            } else if running {
                running = false;
                Some(current)
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct Path {
    pub points: Vec<Point>,
}

impl Path {
    pub fn lines(&self) -> impl Iterator<Item = Line> + '_ {
        let mut points = self.points.iter();

        let mut start = points.next().cloned();

        std::iter::from_fn(move || {
            if let Some(&end) = points.next() {
                let line = Line {
                    start: start.unwrap(),
                    end,
                };
                start = Some(end);
                Some(line)
            } else {
                None
            }
        })
    }
}

impl FromStr for Path {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let points = s
            .split(" -> ")
            .map(|point| point.parse())
            .collect::<eyre::Result<Vec<Point>>>()?;

        Ok(Self { points })
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.points.iter().join_with(" -> "))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub min: Point,
    pub max: Point,
}

impl Bounds {
    pub fn new(point: Point) -> Self {
        Self {
            min: point,
            max: point,
        }
    }

    pub fn add(&mut self, point: Point) {
        self.min.x = std::cmp::min(self.min.x, point.x);
        self.min.y = std::cmp::min(self.min.y, point.y);
        self.max.x = std::cmp::max(self.max.x, point.x);
        self.max.y = std::cmp::max(self.max.y, point.y);
    }

    pub fn x_bounds(&self) -> RangeInclusive<i32> {
        self.min.x..=self.max.x
    }

    pub fn y_bounds(&self) -> RangeInclusive<i32> {
        self.min.y..=self.max.y
    }

    pub fn contains(&self, point: Point) -> bool {
        self.x_bounds().contains(&point.x) && self.y_bounds().contains(&point.y)
    }

    pub fn width(&self) -> i32 {
        (self.max.x - self.min.x) + 1
    }

    pub fn height(&self) -> i32 {
        (self.max.y - self.min.y) + 1
    }

    pub fn bottom_left(&self) -> Point {
        let x = self.min.x;
        let y = self.max.y;

        Point { x, y }
    }

    pub fn bottom_right(&self) -> Point {
        let x = self.max.x;
        let y = self.max.y;

        Point { x, y }
    }
}
