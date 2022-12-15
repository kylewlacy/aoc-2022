use std::ops::RangeInclusive;

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

    pub fn union(&mut self, bounds: &Bounds) {
        self.min.x = std::cmp::min(self.min.x, bounds.min.x);
        self.max.x = std::cmp::max(self.max.x, bounds.max.x);
        self.min.y = std::cmp::min(self.min.y, bounds.min.y);
        self.max.y = std::cmp::max(self.max.y, bounds.max.y);
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

    pub fn points_row(&self, row: i32) -> impl Iterator<Item = Point> {
        self.x_bounds().map(move |x| Point { x, y: row })
    }
}
