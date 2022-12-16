use std::{collections::HashSet, io::BufRead, str::FromStr};

use clap::Parser;
use day15::{Bounds, Point};
use itertools::Itertools;

#[derive(Parser)]
struct Args {
    #[clap(long)]
    max_bounds: i32,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let stdin = std::io::stdin().lock();
    let sensor_reports = stdin
        .lines()
        .map(|line| eyre::Result::Ok(line?.parse::<SensorReport>()?))
        .collect::<eyre::Result<Vec<_>>>()?;

    let bounds = Bounds {
        min: Point { x: 0, y: 0 },
        max: Point {
            x: args.max_bounds,
            y: args.max_bounds,
        },
    };

    let report_pairs = sensor_reports
        .iter()
        .permutations(2)
        .map(|pair| -> [_; 2] { pair.try_into().unwrap() });
    let candidate_points = report_pairs.flat_map(|[a, b]| {
        let a_edge: HashSet<_> = a
            .outer_edge_points()
            .filter(|&point| bounds.contains(point))
            .collect();
        let b_edge: HashSet<_> = b
            .outer_edge_points()
            .filter(|&point| bounds.contains(point))
            .collect();

        a_edge.intersection(&b_edge).cloned().collect::<Vec<_>>()
    });

    for point in candidate_points {
        if sensor_reports
            .iter()
            .all(|report| !report.covers_point(point) && report.closest_beacon != point)
        {
            println!("Found beacon: {point:?}");
            println!("Tuning frequency: {}", tuning_frequency(point));
            return Ok(());
        }
    }

    eyre::bail!("point not found");
}

#[derive(Debug)]
struct SensorReport {
    sensor: Point,
    closest_beacon: Point,
}

impl SensorReport {
    fn covers_point(&self, point: Point) -> bool {
        let sensor_radius = self.sensor.manhattan_distance(&self.closest_beacon);
        let distance = self.sensor.manhattan_distance(&point);

        sensor_radius >= distance
    }

    fn outer_edge_points(&self) -> impl Iterator<Item = Point> {
        let sensor_radius = self.sensor.manhattan_distance(&self.closest_beacon);
        let top = Point {
            x: self.sensor.x,
            y: self.sensor.y + sensor_radius + 1,
        };
        let right = Point {
            x: self.sensor.x + sensor_radius + 1,
            y: self.sensor.y,
        };
        let bottom = Point {
            x: self.sensor.x,
            y: self.sensor.y - sensor_radius - 1,
        };
        let left = Point {
            x: self.sensor.x - sensor_radius - 1,
            y: self.sensor.y,
        };

        walk_points(top, right, (1, -1))
            .chain(walk_points(right, bottom, (-1, -1)))
            .chain(walk_points(bottom, left, (-1, 1)))
            .chain(walk_points(left, top, (1, 1)))
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

fn walk_points(start: Point, end: Point, walk: (i32, i32)) -> impl Iterator<Item = Point> {
    let mut current = start;
    let (walk_x, walk_y) = walk;

    let mut is_running = true;
    std::iter::from_fn(move || {
        if !is_running {
            None
        } else if current == end {
            is_running = false;
            Some(current)
        } else {
            let last = current;
            current = Point {
                x: current.x + walk_x,
                y: current.y + walk_y,
            };
            Some(last)
        }
    })
}

fn tuning_frequency(point: Point) -> i64 {
    let x: i64 = point.x.into();
    let y: i64 = point.y.into();
    (x * 4_000_000) + y
}
