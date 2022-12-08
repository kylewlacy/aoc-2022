use std::io::BufRead;

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let mut tree_patch = TreePatch::new();

    let stdin = std::io::stdin().lock();
    for line in stdin.lines() {
        let line = line?;
        tree_patch.parse_row(&line)?;
    }

    let total_visible_trees = tree_patch
        .indices()
        .filter(|&index| tree_patch.is_visible(index))
        .count();
    println!("{total_visible_trees}");

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

    fn visibility_candidates(
        &self,
        index: usize,
        direction: Direction,
    ) -> impl Iterator<Item = usize> + '_ {
        let (mut row, mut col) = self.location(index);
        let (row_stride, col_stride) = match direction {
            Direction::TopToBottom => (-1, 0),
            Direction::BottomToTop => (1, 0),
            Direction::LeftToRight => (0, -1),
            Direction::RightToLeft => (0, 1),
        };

        std::iter::from_fn(move || {
            let _row = row;
            let _col = col;

            row += row_stride;
            col += col_stride;

            let index_ = self.index((row, col));

            index_
        })
        .collect::<Vec<_>>()
        .into_iter()
    }

    fn is_visible_from(&self, index: usize, direction: Direction) -> bool {
        self.visibility_candidates(index, direction)
            .all(|candidate_index| self.trees[candidate_index].height < self.trees[index].height)
    }

    fn is_visible(&self, index: usize) -> bool {
        DIRECTIONS
            .iter()
            .any(|&direction| self.is_visible_from(index, direction))
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

const DIRECTIONS: [Direction; 4] = [
    Direction::TopToBottom,
    Direction::BottomToTop,
    Direction::LeftToRight,
    Direction::RightToLeft,
];

#[test]
fn test_index_to_location_mapping() {
    let trees = TreePatch::from_rows([[0, 1, 2, 3], [4, 5, 6, 7], [8, 9, 0, 1]]);
    let expected_locations = [
        (0, 0),
        (0, 1),
        (0, 2),
        (0, 3),
        (1, 0),
        (1, 1),
        (1, 2),
        (1, 3),
        (2, 0),
        (2, 1),
        (2, 2),
        (2, 3),
    ];

    assert_eq!(trees.indices().count(), expected_locations.len());
    for (index, expected_location) in trees.indices().zip(expected_locations) {
        let actual_location = trees.location(index);
        assert_eq!(expected_location, actual_location);
        assert_eq!(trees.index(actual_location), Some(index));
    }
}

#[test]
fn test_visibility_candidates() {
    let trees = TreePatch::from_rows([[1, 1, 1, 1], [1, 1, 1, 1], [1, 1, 1, 1]]);

    let visibility_candidates_at =
        |loc: (isize, isize), direction: Direction| -> Vec<(isize, isize)> {
            let index = trees.index(loc).expect("invalid index");
            let mut candidates = trees
                .visibility_candidates(index, direction)
                .map(|index| trees.location(index))
                .collect::<Vec<_>>();
            candidates.sort();
            candidates
        };

    // Visibility of column 0 from top
    assert_eq!(
        visibility_candidates_at((0, 0), Direction::TopToBottom),
        vec![],
    );
    assert_eq!(
        visibility_candidates_at((1, 0), Direction::TopToBottom),
        vec![(0, 0)]
    );
    assert_eq!(
        visibility_candidates_at((2, 0), Direction::TopToBottom),
        vec![(0, 0), (1, 0)]
    );

    // Visibility of column 1 from top
    assert_eq!(
        visibility_candidates_at((0, 1), Direction::TopToBottom),
        vec![],
    );
    assert_eq!(
        visibility_candidates_at((1, 1), Direction::TopToBottom),
        vec![(0, 1)]
    );
    assert_eq!(
        visibility_candidates_at((2, 1), Direction::TopToBottom),
        vec![(0, 1), (1, 1)]
    );

    // Visibility of column 0 from bottom
    assert_eq!(
        visibility_candidates_at((2, 0), Direction::BottomToTop),
        vec![],
    );
    assert_eq!(
        visibility_candidates_at((1, 0), Direction::BottomToTop),
        vec![(2, 0)]
    );
    assert_eq!(
        visibility_candidates_at((0, 0), Direction::BottomToTop),
        vec![(1, 0), (2, 0)]
    );

    // Visibility of row 0 from left
    assert_eq!(
        visibility_candidates_at((0, 0), Direction::LeftToRight),
        vec![],
    );
    assert_eq!(
        visibility_candidates_at((0, 1), Direction::LeftToRight),
        vec![(0, 0)]
    );
    assert_eq!(
        visibility_candidates_at((0, 2), Direction::LeftToRight),
        vec![(0, 0), (0, 1)]
    );
    assert_eq!(
        visibility_candidates_at((0, 3), Direction::LeftToRight),
        vec![(0, 0), (0, 1), (0, 2)]
    );

    // Visibility of row 0 from right
    assert_eq!(
        visibility_candidates_at((0, 3), Direction::RightToLeft),
        vec![],
    );
    assert_eq!(
        visibility_candidates_at((0, 2), Direction::RightToLeft),
        vec![(0, 3)]
    );
    assert_eq!(
        visibility_candidates_at((0, 1), Direction::RightToLeft),
        vec![(0, 2), (0, 3)]
    );
    assert_eq!(
        visibility_candidates_at((0, 0), Direction::RightToLeft),
        vec![(0, 1), (0, 2), (0, 3)]
    );
}

#[test]
fn test_visibility_from_simple() {
    use std::collections::{HashMap, HashSet};

    let trees = TreePatch::from_rows([[1, 1, 1, 1], [1, 1, 1, 1], [1, 1, 1, 1]]);

    let visible_from_top = HashSet::from([(0, 0), (0, 1), (0, 2), (0, 3)]);
    let visible_from_bottom = HashSet::from([(2, 0), (2, 1), (2, 2), (2, 3)]);
    let visible_from_left = HashSet::from([(0, 0), (1, 0), (2, 0)]);
    let visible_from_right = HashSet::from([(0, 3), (1, 3), (2, 3)]);

    let expected_visibilities = HashMap::from([
        (Direction::TopToBottom, visible_from_top),
        (Direction::BottomToTop, visible_from_bottom),
        (Direction::LeftToRight, visible_from_left),
        (Direction::RightToLeft, visible_from_right),
    ]);

    for direction in DIRECTIONS {
        let expected_visibilities = expected_visibilities.get(&direction).unwrap();

        for index in trees.indices() {
            let location = trees.location(index);

            let expected_visibility = expected_visibilities.contains(&location);
            let actual_visibility = trees.is_visible_from(index, direction);
            assert_eq!(
                expected_visibility,
                actual_visibility,
                "expected {location:?} to be {} from {direction:?}, was {}",
                visiblity_label(expected_visibility),
                visiblity_label(actual_visibility),
            );
        }
    }
    assert!(trees.is_visible_from(0, Direction::TopToBottom));
    assert!(trees.is_visible_from(1, Direction::TopToBottom))
}

#[test]
fn test_visibility_tall_side() {
    use std::collections::HashSet;

    // Check top to bottom
    let tall_top_trees = TreePatch::from_rows([[2, 2, 2, 2], [1, 1, 1, 1], [1, 1, 1, 1]]);
    let visible_from_top = HashSet::from([(0, 0), (0, 1), (0, 2), (0, 3)]);

    for index in tall_top_trees.indices() {
        let location = tall_top_trees.location(index);
        let expected_visibility = visible_from_top.contains(&location);
        let actual_visibility = tall_top_trees.is_visible_from(index, Direction::TopToBottom);
        assert_eq!(
            expected_visibility,
            actual_visibility,
            "expected {location:?} to be {} from top to bottom, was {}",
            visiblity_label(expected_visibility),
            visiblity_label(actual_visibility),
        );
    }

    // Check bottom to top
    let tall_bottom_trees = TreePatch::from_rows([[1, 1, 1, 1], [1, 1, 1, 1], [2, 2, 2, 2]]);
    let visible_from_bottom = HashSet::from([(2, 0), (2, 1), (2, 2), (2, 3)]);

    for index in tall_bottom_trees.indices() {
        let location = tall_bottom_trees.location(index);
        let expected_visibility = visible_from_bottom.contains(&location);
        let actual_visibility = tall_bottom_trees.is_visible_from(index, Direction::BottomToTop);
        assert_eq!(
            expected_visibility,
            actual_visibility,
            "expected {location:?} to be {} from bottom to top, was {}",
            visiblity_label(expected_visibility),
            visiblity_label(actual_visibility),
        );
    }

    // Check left to right
    let tall_left_trees = TreePatch::from_rows([[2, 1, 1, 1], [2, 1, 1, 1], [2, 1, 1, 1]]);
    let visible_from_left = HashSet::from([(0, 0), (1, 0), (2, 0)]);

    for index in tall_left_trees.indices() {
        let location = tall_left_trees.location(index);
        let expected_visibility = visible_from_left.contains(&location);
        let actual_visibility = tall_left_trees.is_visible_from(index, Direction::LeftToRight);
        assert_eq!(
            expected_visibility,
            actual_visibility,
            "expected {location:?} to be {} from left to right, was {}",
            visiblity_label(expected_visibility),
            visiblity_label(actual_visibility),
        );
    }

    // Check right to left
    let tall_right_trees = TreePatch::from_rows([[1, 1, 1, 2], [1, 1, 1, 2], [1, 1, 1, 2]]);
    let visible_from_right = HashSet::from([(0, 3), (1, 3), (2, 3)]);

    for index in tall_right_trees.indices() {
        let location = tall_right_trees.location(index);
        let expected_visibility = visible_from_right.contains(&location);
        let actual_visibility = tall_right_trees.is_visible_from(index, Direction::RightToLeft);
        assert_eq!(
            expected_visibility,
            actual_visibility,
            "expected {location:?} to be {} from right to left, was {}",
            visiblity_label(expected_visibility),
            visiblity_label(actual_visibility),
        );
    }
}

#[cfg(test)]
fn visiblity_label(visible: bool) -> &'static str {
    match visible {
        true => "visibile",
        false => "invisible",
    }
}
