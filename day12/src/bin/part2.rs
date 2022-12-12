use pathfinding::directed::dijkstra::dijkstra;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let stdin = std::io::stdin().lock();
    let grid = Grid::parse(stdin)?;

    // for row in 0..(grid.height()) {
    //     for col in 0..(grid.width()) {}
    // }

    let fewest_steps = grid.find_fewest_steps()?;

    println!("{fewest_steps}");

    Ok(())
}

#[derive(Debug, Clone)]
struct Grid {
    cell_heights: Vec<u8>,
    width: usize,
    peaks: Vec<Position>,
    end: Position,
}

impl Grid {
    fn parse(reader: impl std::io::BufRead) -> eyre::Result<Self> {
        let mut cell_heights = vec![];
        let mut width = None;
        let mut peaks = vec![];
        let mut end = None;
        for (row, line) in reader.lines().enumerate() {
            let line = line?;

            match width {
                Some(width) => {
                    eyre::ensure!(
                        width == line.len(),
                        "expected line to match width {width}, but was {}",
                        line.len()
                    );
                }
                None => {
                    width = Some(line.len());
                }
            }

            for (col, byte) in line.bytes().enumerate() {
                let position = Position { row, col };

                match byte {
                    b'a' | b'S' => {
                        cell_heights.push(0);
                        peaks.push(position)
                    }
                    b'E' => {
                        cell_heights.push(25);
                        let old_end = end.replace(position);

                        if let Some(old_end) = old_end {
                            eyre::bail!("found multiple end points at {old_end:?} and {end:?}");
                        }
                    }
                    height @ b'b'..=b'z' => {
                        cell_heights.push(height - b'a');
                    }
                    other => {
                        eyre::bail!("could not parse byte {} at ({position:?})", other)
                    }
                }
            }
        }

        let width = width.ok_or_else(|| eyre::eyre!("width not found"))?;
        let end = end.ok_or_else(|| eyre::eyre!("end not set"))?;

        Ok(Self {
            cell_heights,
            width,
            peaks,
            end,
        })
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.cell_heights.len() / self.width
    }

    fn successors(&self, position: Position) -> eyre::Result<impl Iterator<Item = Position> + '_> {
        let current_height = self
            .height_at(position)
            .ok_or_else(|| eyre::eyre!("could not get height at position {position:?}"))?;
        let candidates = CANDIDATE_OFFSETS
            .iter()
            .flat_map(move |&offset| self.offset(position, offset));
        let successors = candidates.filter(move |&position| {
            let height = self.height_at(position).expect("out of bounds candidate");
            height <= current_height + 1
        });

        Ok(successors.collect::<Vec<_>>().into_iter())
    }

    fn height_at(&self, position: Position) -> Option<u8> {
        let index = self.index(position)?;
        let height = self.cell_heights[index];
        Some(height)
    }

    fn index(&self, position: Position) -> Option<usize> {
        let width = self.width();
        let height = self.height();

        if position.row < height && position.col < width {
            let index = (position.row * width) + position.col;
            let index = index.try_into().ok()?;

            assert!(index < self.cell_heights.len());

            Some(index)
        } else {
            None
        }
    }

    fn offset(&self, position: Position, offset: (isize, isize)) -> Option<Position> {
        let (offset_row, offset_col) = offset;

        let row: isize = position.row.try_into().ok()?;
        let col: isize = position.col.try_into().ok()?;

        let width: isize = self.width().try_into().ok()?;
        let height: isize = self.height().try_into().ok()?;

        let new_row = row + offset_row;
        let new_col = col + offset_col;

        if !(0..height).contains(&new_row) || !(0..width).contains(&new_col) {
            return None;
        }

        Some(Position {
            row: new_row.try_into().unwrap(),
            col: new_col.try_into().unwrap(),
        })
    }

    fn find_fewest_steps(&self) -> eyre::Result<usize> {
        let fewest_steps = self
            .peaks
            .iter()
            .filter_map(|&peak| self.find_fewest_steps_from(peak))
            .min();

        let fewest_steps =
            fewest_steps.ok_or_else(|| eyre::eyre!("no paths found for any peaks"))?;

        Ok(fewest_steps)
    }

    fn find_fewest_steps_from(&self, start: Position) -> Option<usize> {
        let path = dijkstra(
            &start,
            move |&pos| {
                self.successors(pos)
                    .unwrap()
                    .map(|successor| (successor, 1))
            },
            move |&pos| pos == self.end,
        );

        let (path, _) = path?;

        // Subtract 1 to get the number of movements required
        let fewest_steps = path.len() - 1;

        Some(fewest_steps)
    }
}

const CANDIDATE_OFFSETS: [(isize, isize); 4] = [
    // Up
    (0, 1),
    // Right
    (1, 0),
    // Down
    (0, -1),
    // Left
    (-1, 0),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    row: usize,
    col: usize,
}
