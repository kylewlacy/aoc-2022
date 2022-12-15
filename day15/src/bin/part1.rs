use std::{io::BufRead, str::FromStr};

use clap::Parser;
use day15::{Bounds, Point};

#[derive(Parser)]
struct Args {
    #[clap(long)]
    search_row: i32,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let stdin = std::io::stdin().lock();
    let sensor_reports = stdin
        .lines()
        .map(|line| eyre::Result::Ok(line?.parse::<SensorReport>()?))
        .collect::<eyre::Result<Vec<_>>>()?;

    let initial_bounds: Option<Bounds> = None;
    let bounds = sensor_reports
        .iter()
        .fold(initial_bounds, |bounds, report| match bounds {
            Some(mut bounds) => {
                bounds.union(&report.covered_bounds());
                Some(bounds)
            }
            None => Some(report.covered_bounds()),
        });

    let bounds = bounds.unwrap_or_else(|| Bounds::new(Point { x: 0, y: 0 }));

    let num_beaconless_points = bounds
        .points_row(args.search_row)
        .filter(|&point| is_beaconless(&sensor_reports, point))
        .count();

    println!("Total beaconless points: {num_beaconless_points}");

    Ok(())
}

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

    fn covered_bounds(&self) -> Bounds {
        let sensor_radius = self.sensor.manhattan_distance(&self.closest_beacon);
        let min_x = self.sensor.x - sensor_radius;
        let max_x = self.sensor.x + sensor_radius;
        let min_y = self.sensor.y - sensor_radius;
        let max_y = self.sensor.y + sensor_radius;

        Bounds {
            min: Point { x: min_x, y: min_y },
            max: Point { x: max_x, y: max_y },
        }
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

fn is_beaconless<'a>(
    sensor_reports: impl IntoIterator<Item = &'a SensorReport>,
    point: Point,
) -> bool {
    for report in sensor_reports {
        if report.closest_beacon == point {
            return false;
        } else if report.covers_point(point) {
            return true;
        }
    }
    return false;
}
