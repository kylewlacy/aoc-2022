use std::{fmt::Display, io::BufRead, str::FromStr};

use clap::Parser;
use day15::{Bounds, Point};
use joinery::JoinableIterator;

#[derive(Parser)]
struct Args {
    #[clap(long)]
    search_row: i32,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let stdin = std::io::stdin().lock();
    let mut grid = None;
    for line in stdin.lines() {
        let line = line?;
        let report = line.parse::<SensorReport>()?;

        let grid =
            grid.get_or_insert_with(|| Grid::new(Cell::default(), Bounds::new(report.sensor)));

        grid.update(report.sensor, |cell| cell.kind = CellKind::Sensor);
        grid.update(report.closest_beacon, |cell| cell.kind = CellKind::Beacon);

        for point in report.covered_points() {
            grid.update(point, |cell| cell.is_covered = true);
        }
    }

    let grid =
        grid.unwrap_or_else(|| Grid::new(Cell::default(), Bounds::new(Point { x: 0, y: 0 })));

    let num_covered_points = grid
        .row(args.search_row)
        .filter(|&(_, cell)| cell.is_beaconless())
        .count();

    println!("{}", grid.display());

    println!("Total covered points: {num_covered_points}");

    Ok(())
}

struct SensorReport {
    sensor: Point,
    closest_beacon: Point,
}

impl SensorReport {
    fn covered_points(&self) -> impl Iterator<Item = Point> {
        let radius = self.sensor.manhattan_distance(&self.closest_beacon);

        let sensor = self.sensor;
        let x_min = sensor.x - radius;
        let x_max = sensor.x + radius;
        let y_min = sensor.y - radius;
        let y_max = sensor.y + radius;

        (x_min..=x_max)
            .flat_map(move |x| (y_min..=y_max).map(move |y| Point { x, y }))
            .filter(move |point| point.manhattan_distance(&sensor) <= radius)
    }
}

impl FromStr for SensorReport {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let caps = SENSOR_REPORT_REGEX
            .captures(s)
            .ok_or_else(|| eyre::eyre!("invalid report: {}", s))?;

        let sensor_x = caps["sensor_x"].parse()?;
        let sensor_y = caps["sensor_y"].parse()?;
        let beacon_x = caps["beacon_x"].parse()?;
        let beacon_y = caps["beacon_y"].parse()?;

        let sensor = Point {
            x: sensor_x,
            y: sensor_y,
        };
        let closest_beacon = Point {
            x: beacon_x,
            y: beacon_y,
        };

        Ok(Self {
            sensor,
            closest_beacon,
        })
    }
}

lazy_static::lazy_static! {
    static ref SENSOR_REPORT_REGEX: regex::Regex = regex::Regex::new(
        r"^Sensor at x=(?P<sensor_x>-?\d+), y=(?P<sensor_y>-?\d+): closest beacon is at x=(?P<beacon_x>-?\d+), y=(?P<beacon_y>-?\d+)$",
    ).unwrap();
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Cell {
    kind: CellKind,
    is_covered: bool,
}

impl Cell {
    fn is_beaconless(&self) -> bool {
        self.is_covered && self.kind == CellKind::Empty
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum CellKind {
    #[default]
    Empty,
    Beacon,
    Sensor,
}

struct Grid {
    bounds: Bounds,
    cells: Vec<Cell>,
}

impl Grid {
    fn new(cell: Cell, bounds: Bounds) -> Self {
        let num_cells = bounds.width() * bounds.height();
        let num_cells = num_cells.try_into().unwrap();
        let cells = vec![cell; num_cells];

        Self { bounds, cells }
    }

    fn grow(&mut self, bounds: Bounds) {
        let new_bounds = self.bounds.union(&bounds);

        if new_bounds == self.bounds {
            return;
        }

        let mut new_grid = Grid::new(Cell::default(), new_bounds);

        for (point, cell) in self.iter() {
            let new_offset = new_grid.offset(point).unwrap();
            new_grid.cells[new_offset] = cell;
        }

        *self = new_grid;
    }

    fn offset(&self, point: Point) -> Option<usize> {
        if !self.bounds.contains(point) {
            return None;
        }

        let row = point.x - self.bounds.min.x;
        let col = point.y - self.bounds.min.y;

        let offset = (col * self.bounds.width()) + row;
        let offset = offset.try_into().unwrap();

        Some(offset)
    }

    fn try_get(&self, point: Point) -> Option<Cell> {
        let offset = self.offset(point)?;
        Some(self.cells[offset])
    }

    fn get(&self, point: Point) -> Cell {
        self.try_get(point).unwrap_or_default()
    }

    fn update(&mut self, point: Point, f: impl FnOnce(&mut Cell)) {
        self.grow(Bounds::new(point));
        let offset = self.offset(point).unwrap();
        let cell = &mut self.cells[offset];
        f(cell);
    }

    fn iter(&self) -> impl Iterator<Item = (Point, Cell)> + '_ {
        self.bounds.points().map(|point| (point, self.get(point)))
    }

    fn row(&self, row: i32) -> impl Iterator<Item = (Point, Cell)> + '_ {
        self.bounds.x_bounds().map(move |x| {
            let point = Point { x, y: row };
            (point, self.get(point))
        })
    }

    fn display(&self) -> impl Display + '_ {
        self.bounds
            .y_bounds()
            .map(move |y| {
                let row = self
                    .bounds
                    .x_bounds()
                    .map(move |x| {
                        let point = Point { x, y };

                        let cell = self.get(point);
                        match cell {
                            Cell {
                                kind: CellKind::Beacon,
                                ..
                            } => 'B',
                            Cell {
                                kind: CellKind::Sensor,
                                ..
                            } => 'S',
                            Cell {
                                is_covered: true, ..
                            } => '#',
                            Cell {
                                kind: CellKind::Empty,
                                ..
                            } => '.',
                        }
                    })
                    .join_concat();

                lazy_format::lazy_format!("{y:3} {row}")
            })
            .join_with("\n")
    }
}
