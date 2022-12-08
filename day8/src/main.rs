use std::io::BufRead;

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let mut tree_patch = TreePatch::new();

    let stdin = std::io::stdin().lock();
    for line in stdin.lines() {
        let line = line?;
        tree_patch.parse_row(&line)?;
    }

    let best_scenic_score = tree_patch
        .indices()
        .map(|index| tree_patch.scenic_score(index))
        .max()
        .unwrap_or_default();
    println!("{best_scenic_score}");

    Ok(())
}

struct TreePatch {
    width: usize,
    trees: Vec<Tree>,
}

impl TreePatch {
    fn new() -> Self {
        Self {
            width: 0,
            trees: vec![],
        }
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.trees.len() / self.width()
    }

    #[cfg(test)]
    fn from_rows<const N: usize, const M: usize>(rows: [[u8; M]; N]) -> Self {
        let width = rows.get(0).map(|row| row.len()).unwrap_or_default();
        let mut tree_patch = Self {
            width,
            trees: vec![],
        };
        for row in rows.iter() {
            tree_patch
                .trees
                .extend(row.iter().map(|&height| Tree { height }));
        }

        tree_patch
    }

    fn parse_row(&mut self, row: &str) -> anyhow::Result<()> {
        match self.width {
            0 => {
                self.width = row.len();
            }
            _ => {
                anyhow::ensure!(self.width == row.len());
            }
        }

        let mut row = row
            .chars()
            .map(|height| Tree::parse_cell(height))
            .collect::<anyhow::Result<Vec<_>>>()?;
        self.trees.append(&mut row);

        Ok(())
    }

    fn indices(&self) -> impl Iterator<Item = usize> {
        0..self.trees.len()
    }

    fn location(&self, index: usize) -> (isize, isize) {
        let row = index / self.width;
        let col = index % self.width;

        let row = row.try_into().expect("row overflow");
        let col = col.try_into().expect("col overflow");

        (row, col)
    }

    fn index(&self, location: (isize, isize)) -> Option<usize> {
        let width = self.width();
        let height = self.height();

        let (row, col) = location;
        let row: usize = row.try_into().ok()?;
        let col: usize = col.try_into().ok()?;

        if row < height && col < width {
            let index = (row * width) + col;
            let index = index.try_into().ok()?;

            assert!(index < self.trees.len());

            Some(index)
        } else {
            None
        }
    }

    fn scenic_score_for_direction(&self, index: usize, direction: Direction) -> u64 {
        let (mut row, mut col) = self.location(index);
        let (row_stride, col_stride) = direction.stride();

        let mut score = 0;
        loop {
            row += row_stride;
            col += col_stride;
            let candidate_index = match self.index((row, col)) {
                Some(index) => index,
                None => {
                    break;
                }
            };

            let candidate_height = self.trees[candidate_index].height;

            score += 1;

            if candidate_height >= self.trees[index].height {
                break;
            }
        }

        score
    }

    fn scenic_score(&self, index: usize) -> u64 {
        DIRECTIONS
            .into_iter()
            .map(|direction| self.scenic_score_for_direction(index, direction))
            .product()
    }
}

struct Tree {
    height: u8,
}

impl Tree {
    fn new(height: u8) -> Self {
        assert!(height <= 9, "invalid tree height: {height}");
        Self { height }
    }

    fn parse_cell(height: char) -> anyhow::Result<Self> {
        let height: u32 = height.to_digit(10).context("invalid tree height")?;
        anyhow::ensure!(height <= 9);

        let height: u8 = height.try_into().unwrap();
        Ok(Self::new(height))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    TopToBottom,
    BottomToTop,
    LeftToRight,
    RightToLeft,
}

impl Direction {
    fn stride(&self) -> (isize, isize) {
        match self {
            Direction::TopToBottom => (-1, 0),
            Direction::BottomToTop => (1, 0),
            Direction::LeftToRight => (0, -1),
            Direction::RightToLeft => (0, 1),
        }
    }
}

const DIRECTIONS: [Direction; 4] = [
    Direction::TopToBottom,
    Direction::BottomToTop,
    Direction::LeftToRight,
    Direction::RightToLeft,
];
