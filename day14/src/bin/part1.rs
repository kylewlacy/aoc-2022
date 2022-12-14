use std::{
    fmt::Display,
    io::BufRead,
    ops::{Index, IndexMut},
};

use clap::Parser;
use day14::{Bounds, Path, Point, Vector};
use eyre::ContextCompat;
use joinery::JoinableIterator;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    display: bool,
    #[clap(short, long, default_value_t = 50)]
    rate: u64,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let stdin = std::io::stdin().lock();

    let paths = stdin
        .lines()
        .map(|line| line?.parse())
        .collect::<eyre::Result<Vec<Path>>>()?;

    let mut world = World::new(STARTING_POINT, &paths);

    if args.display {
        println!("{}", termion::clear::All);
    }

    let mut steps = 0;
    loop {
        if args.display {
            println!(
                "{}{}Steps: {steps}\n{}",
                termion::cursor::Goto(1, 1),
                termion::clear::CurrentLine,
                world.display(),
            );
            std::thread::sleep(std::time::Duration::from_millis(args.rate));
        }

        let is_running = world.step();
        if !is_running {
            break;
        }

        steps += 1;
    }

    println!("Total steps: {steps}\n{}", world.display());

    let resting_sand = world
        .cells
        .iter()
        .filter(|&(_, cell)| cell == Cell::SettledSand)
        .count();
    println!("Resting sand: {resting_sand}");

    Ok(())
}

const STARTING_POINT: Point = Point { x: 500, y: 0 };

struct World {
    cells: Cells,
    source: Point,
}

impl World {
    fn new(source: Point, paths: &[Path]) -> Self {
        let mut bounds = Bounds::new(source);

        for path in paths {
            for &point in &path.points {
                bounds.add(point);
            }
        }

        let mut cells = Cells::new(Cell::Air, bounds);

        for path in paths {
            for line in path.lines() {
                for point in line.points() {
                    cells[point] = Cell::Rock;
                }
            }
        }

        Self { cells, source }
    }

    fn display(&self) -> impl Display + '_ {
        let ys = self.cells.bounds.y_bounds();

        ys.map(move |y| {
            let xs = self.cells.bounds.x_bounds();

            xs.map(move |x| {
                let point = Point { x, y };

                if point == self.source {
                    '+'
                } else {
                    match self.cells[point] {
                        Cell::Air => '.',
                        Cell::Rock => '#',
                        Cell::FallingSand => '~',
                        Cell::SettledSand => 'o',
                    }
                }
            })
            .join_concat()
        })
        .join_with("\n")
    }

    fn step(&mut self) -> bool {
        let falling_sand = self
            .cells
            .iter()
            .find(|&(_, cell)| cell == Cell::FallingSand);

        match falling_sand {
            Some((current_sand_point, current_sand_cell)) => {
                let mut new_point: Option<Point> = None;

                for falling_vector in FALLING_SAND_VECTORS {
                    let candidate_point = current_sand_point + falling_vector;
                    match self.cells.get(candidate_point) {
                        Some(Cell::Air) => {
                            new_point = Some(candidate_point);
                            break;
                        }
                        Some(Cell::Rock | Cell::FallingSand | Cell::SettledSand) => {}
                        None => {
                            // Next position doesn't exist, so sand flowed out of bounds.
                            return false;
                        }
                    }
                }

                match new_point {
                    Some(new_point) => {
                        self.cells[new_point] = current_sand_cell;
                        self.cells[current_sand_point] = Cell::Air;
                    }
                    None => {
                        self.cells[current_sand_point] = Cell::SettledSand;
                    }
                }
            }
            None => {
                self.cells[self.source] = Cell::FallingSand;
            }
        }

        true
    }
}

const FALLING_SAND_VECTORS: [Vector; 3] = [
    Vector { x: 0, y: 1 },
    Vector { x: -1, y: 1 },
    Vector { x: 1, y: 1 },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Air,
    Rock,
    FallingSand,
    SettledSand,
}

struct Cells {
    bounds: Bounds,
    cells: Vec<Cell>,
}

impl Cells {
    fn new(cell: Cell, bounds: Bounds) -> Self {
        let num_cells = bounds.width() * bounds.height();
        let num_cells = num_cells.try_into().unwrap();
        let cells = vec![cell; num_cells];

        Self { bounds, cells }
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

    fn get(&self, point: Point) -> Option<&Cell> {
        let offset = self.offset(point)?;
        Some(&self.cells[offset])
    }

    fn get_mut(&mut self, point: Point) -> Option<&mut Cell> {
        let offset = self.offset(point)?;
        Some(&mut self.cells[offset])
    }

    fn iter(&self) -> impl Iterator<Item = (Point, Cell)> + '_ {
        let ys = self.bounds.y_bounds();

        ys.flat_map(move |y| {
            let xs = self.bounds.x_bounds();

            xs.map(move |x| {
                let point = Point { x, y };
                let cell = self[point];
                (point, cell)
            })
        })
    }
}

impl Index<Point> for Cells {
    type Output = Cell;

    fn index(&self, point: Point) -> &Cell {
        let bounds = self.bounds;
        self.get(point)
            .with_context(|| format!("point {point} was out of bounds {bounds:?}"))
            .unwrap()
    }
}

impl IndexMut<Point> for Cells {
    fn index_mut(&mut self, point: Point) -> &mut Cell {
        let bounds = self.bounds;
        self.get_mut(point)
            .with_context(|| format!("point {point} was out of bounds {bounds:?}"))
            .unwrap()
    }
}
