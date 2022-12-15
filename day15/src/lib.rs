use std::{
    fmt::Display,
    ops::{Add, AddAssign, RangeInclusive, Sub},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn manhattan_distance(&self, other: &Point) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    pub fn points(&self) -> impl Iterator<Item = Point> {
        let min_x = self.min.x;
        let max_x = self.max.x;
        let min_y = self.min.y;
        let max_y = self.max.y;
        (min_x..=max_x).flat_map(move |x| (min_y..=max_y).map(move |y| Point { x, y }))
    }

    pub fn union(&self, bounds: &Bounds) -> Self {
        let min_x = std::cmp::min(self.min.x, bounds.min.x);
        let max_x = std::cmp::max(self.max.x, bounds.max.x);
        let min_y = std::cmp::min(self.min.y, bounds.min.y);
        let max_y = std::cmp::max(self.max.y, bounds.max.y);

        Self {
            min: Point { x: min_x, y: min_y },
            max: Point { x: max_x, y: max_y },
        }
    }
}
