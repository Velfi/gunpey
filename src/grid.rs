use crate::cell::Cell;
use crate::{
    error::GunpeyLibError,
    grid_pos::GridPos,
    line_fragment::{LineFragment, LineFragmentKind},
};
use druid::{im::Vector, Data};
use itertools::Itertools;
use log::debug;
use std::collections::HashSet;

#[derive(Debug, Clone, Data, PartialEq)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub cells: Vector<Cell>,
}

type CharGrid = Vec<Vec<char>>;
type Bitmask = Vec<Vec<u8>>;

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width > 0, "width of new Grid must be greater than 0!");
        assert!(height > 1, "height of new Grid must be greater than 1!");

        let length = width * height;
        let cells = (0..length).map(|_| Cell::Empty).collect();

        Self {
            width,
            height,
            cells,
        }
    }

    pub fn new_from_chars(chars: CharGrid) -> Self {
        let width = chars[0].len();
        let height = chars.len();
        assert!(width > 0, "width of new Grid must be greater than 0!");
        assert!(height > 1, "height of new Grid must be greater than 1!");

        let cells = chars
            .iter()
            .flat_map(|x| x.iter().map(Cell::from_char))
            .collect();

        Self {
            width,
            height,
            cells,
        }
    }

    pub fn as_chars(&self) -> CharGrid {
        self.cells
            .iter()
            .chunks(self.width)
            .into_iter()
            .map(|x| x.into_iter().map(Cell::to_char).collect())
            .collect()
    }

    pub fn as_active_bitmask(&self) -> Bitmask {
        self.cells
            .iter()
            .chunks(self.width)
            .into_iter()
            .map(|x| x.into_iter().map(|cell| cell.is_active() as u8).collect())
            .collect()
    }

    pub fn swap_cells(
        &mut self,
        cell_pos_a: GridPos,
        cell_pos_b: GridPos,
    ) -> Result<(), GunpeyLibError> {
        if cell_pos_a == cell_pos_b {
            return Err(GunpeyLibError::CantSwapSamePositon(cell_pos_a, cell_pos_b));
        }

        match (
            self.get_index_from_pos(cell_pos_a),
            self.get_index_from_pos(cell_pos_b),
        ) {
            (Some(cell_index_a), Some(cell_index_b)) => {
                let res = self.swap_cells_by_index(cell_index_a, cell_index_b);
                if res.is_ok() {
                    self.recalculate_active_cells()
                }
                res
            }
            _ => Err(GunpeyLibError::CantSwapBadPosition(cell_pos_a, cell_pos_b)),
        }
    }

    fn get_index_from_pos(&self, GridPos { x, y }: GridPos) -> Option<usize> {
        let i = y + self.width * x;

        (0..self.cells.len()).contains(&i).then(|| i)
    }

    fn get_pos_from_index(&self, index: usize) -> GridPos {
        get_pos_from_index(index, self.width)
    }

    // 1) Create a node graph of every connected cell
    // 2) If no node connects to the opposite side, the graph is dead
    //      for a group of nodes, at least one node must be connected to the left edge
    //                        and at least one node must be connected to the right edge
    // 3) Every node with exactly one connection and no edge connection is dead, remove from list
    // 4) Every remaining node is live
    fn recalculate_active_cells(&mut self) {
        let edges = self.edges();

        debug!("edges\n{:?}", edges);

        let mut is_connected_to_left_edge: HashSet<GridPos> = (0..self.height)
            .map(|x| GridPos { x, y: 0 })
            .filter(|gp| !self.is_cell_empty(*gp))
            .collect();

        loop {
            let mut change = false;

            for &connected_gp in is_connected_to_left_edge.clone().iter() {
                for (gp1, gp2) in edges.iter() {
                    if *gp1 == connected_gp {
                        change = is_connected_to_left_edge.insert(*gp2);
                    } else if *gp2 == connected_gp {
                        change = is_connected_to_left_edge.insert(*gp1);
                    }
                }
            }

            if change == false {
                break;
            }
        }

        let mut is_connected_to_right_edge: HashSet<GridPos> = (0..self.height)
            .map(|x| GridPos {
                x,
                y: self.width - 1,
            })
            .filter(|gp| !self.is_cell_empty(*gp))
            .collect();

        loop {
            let mut change = false;

            for &connected_gp in is_connected_to_right_edge.clone().iter() {
                for (gp1, gp2) in edges.iter() {
                    if *gp1 == connected_gp {
                        change = change || is_connected_to_right_edge.insert(*gp2);
                    } else if *gp2 == connected_gp {
                        change = change || is_connected_to_right_edge.insert(*gp1);
                    }
                }
            }

            if change == false {
                break;
            }
        }

        debug!("is_connected_to_left_edge\n{:?}", is_connected_to_left_edge);
        debug!(
            "is_connected_to_right_edge\n{:?}",
            is_connected_to_right_edge
        );

        let active_grid_positions: HashSet<_> = is_connected_to_left_edge
            .union(&is_connected_to_right_edge)
            .collect();

        let width = self.width;
        self.cells
            .iter_mut()
            // must enumerate before we ignore empty cells because otherwise the indexes will be wrong
            .enumerate()
            // Empty cells have no edges so we ignore them
            .filter(|(_, &mut c)| !c.is_empty())
            .for_each(|(index, cell)| {
                let gp1 = get_pos_from_index(index, width);

                if active_grid_positions.contains(&gp1) {
                    cell.activate()
                } else {
                    cell.deactivate()
                }
            });
    }

    fn edges(&self) -> Vec<(GridPos, GridPos)> {
        let mut edges: Vec<_> = self
            .cells
            .iter()
            // must enumerate before we ignore empty cells because otherwise the indexes will be wrong
            .enumerate()
            // Empty cells have no edges so we ignore them
            .filter(|(_, c)| !c.is_empty())
            .flat_map(|(index, cell)| match cell {
                Cell::Filled(lf) => {
                    let gp1 = self.get_pos_from_index(index);
                    debug!("encountered filled cell at {}, calculating edges", gp1);
                    let possible_edges = match lf.kind {
                        LineFragmentKind::Caret => {
                            vec![
                                self.left(gp1),
                                self.below_left(gp1),
                                self.below(gp1),
                                self.below_right(gp1),
                                self.right(gp1),
                            ]
                        }
                        LineFragmentKind::InvertedCaret => {
                            vec![
                                self.left(gp1),
                                self.above_left(gp1),
                                self.above(gp1),
                                self.above_right(gp1),
                                self.right(gp1),
                            ]
                        }
                        LineFragmentKind::LeftSlash => {
                            vec![
                                self.left(gp1),
                                self.above_left(gp1),
                                self.above(gp1),
                                self.right(gp1),
                                self.below_right(gp1),
                                self.below(gp1),
                            ]
                        }
                        LineFragmentKind::RightSlash => {
                            vec![
                                self.left(gp1),
                                self.below_left(gp1),
                                self.below(gp1),
                                self.above(gp1),
                                self.above_right(gp1),
                                self.right(gp1),
                            ]
                        }
                    };

                    possible_edges
                        .into_iter()
                        .filter_map(move |gp2| gp2)
                        .map(move |gp2| (gp1, gp2))
                }
                // We already filtered out the empties so we good here
                _ => unreachable!(),
            })
            .collect();

        edges.retain(|edge| self.cells_are_connecting(edge));

        edges
    }

    fn cells_are_connecting(&self, (cell_a_pos, cell_b_pos): &(GridPos, GridPos)) -> bool {
        let adjacency = cell_a_pos.adjacency(cell_b_pos);

        match (
            self.get_index_from_pos(*cell_a_pos),
            self.get_index_from_pos(*cell_b_pos),
        ) {
            (Some(cell_a_index), Some(cell_b_index)) => {
                let cell_a = self.cells.get(cell_a_index).unwrap();
                let cell_b = self.cells.get(cell_b_index).unwrap();

                cell_a.is_connected_to(cell_b, adjacency)
            }
            // TODO would it be better to unwrap instead of matching? This should only ever be called with valid grid positions
            _ => false,
        }
    }

    fn swap_cells_by_index(
        &mut self,
        cell_index_a: usize,
        cell_index_b: usize,
    ) -> Result<(), GunpeyLibError> {
        if cell_index_a == cell_index_b {
            return Err(GunpeyLibError::CantSwapSameIndex(
                cell_index_a,
                cell_index_b,
            ));
        }

        let valid_indexes = 0..self.cells.len();

        if valid_indexes.contains(&cell_index_a) && valid_indexes.contains(&cell_index_b) {
            self.cells.swap(cell_index_a, cell_index_b);

            Ok(())
        } else {
            Err(GunpeyLibError::CantSwapBadIndex(cell_index_a, cell_index_b))
        }
    }

    pub fn pop_top_row(&mut self) -> Vector<Cell> {
        debug!("removing top x from grid");
        self.cells.slice(0..self.width)
    }

    pub fn push_bottom_row(&mut self, new_row: Vector<Cell>) -> Result<(), GunpeyLibError> {
        if new_row.len() != self.width {
            return Err(GunpeyLibError::InvalidRowLength(new_row.len(), self.width));
        }

        debug!("pushing new x to bottom of grid");
        self.cells.append(new_row);

        self.recalculate_active_cells();

        Ok(())
    }

    pub fn is_cell_active(&self, cell_pos: GridPos) -> bool {
        let cell_index = self
            .get_index_from_pos(cell_pos)
            .expect("bad grid pos, can't get cell::is_active");
        self.cells
            .get(cell_index)
            .map(|cell| match cell {
                Cell::Filled(LineFragment { is_active, .. }) => *is_active,
                Cell::Empty => false,
            })
            .unwrap_or_default()
    }

    pub fn is_cell_empty(&self, cell_pos: GridPos) -> bool {
        let cell_index = self
            .get_index_from_pos(cell_pos)
            .expect("bad grid pos, can't get cell::is_empty");
        self.cells
            .get(cell_index)
            .map(Cell::is_empty)
            .unwrap_or_default()
    }

    pub fn below(&self, grid_pos: GridPos) -> Option<GridPos> {
        if grid_pos.y == 0 {
            None
        } else {
            Some(GridPos {
                y: grid_pos.y - 1,
                ..grid_pos
            })
        }
    }

    pub fn above(&self, grid_pos: GridPos) -> Option<GridPos> {
        if grid_pos.y == self.height - 1 {
            None
        } else {
            Some(GridPos {
                y: grid_pos.y + 1,
                ..grid_pos
            })
        }
    }

    pub fn right(&self, grid_pos: GridPos) -> Option<GridPos> {
        if grid_pos.x == self.width - 1 {
            None
        } else {
            Some(GridPos {
                x: grid_pos.x + 1,
                ..grid_pos
            })
        }
    }

    pub fn left(&self, grid_pos: GridPos) -> Option<GridPos> {
        if grid_pos.x == 0 {
            None
        } else {
            Some(GridPos {
                x: grid_pos.x - 1,
                ..grid_pos
            })
        }
    }

    pub fn above_right(&self, grid_pos: GridPos) -> Option<GridPos> {
        self.above(grid_pos).and_then(|gp| self.right(gp))
    }

    pub fn below_right(&self, grid_pos: GridPos) -> Option<GridPos> {
        self.below(grid_pos).and_then(|gp| self.right(gp))
    }

    pub fn above_left(&self, grid_pos: GridPos) -> Option<GridPos> {
        self.above(grid_pos).and_then(|gp| self.left(gp))
    }

    pub fn below_left(&self, grid_pos: GridPos) -> Option<GridPos> {
        self.below(grid_pos).and_then(|gp| self.left(gp))
    }
}

fn get_pos_from_index(index: usize, width: usize) -> GridPos {
    let x = index % width;
    let y = index / width;

    GridPos { x, y }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn new_2x2_grid() -> Grid {
        Grid::new_from_chars(vec![vec!['.', '.'], vec!['.', '.']])
    }

    //   0,0 0,1
    //   1,0 1,1

    #[test]
    fn test_grid_pos_above_some() {
        let grid = new_2x2_grid();
        let starting_grid_pos = GridPos { x: 0, y: 0 };
        let expected = Some(GridPos { x: 0, y: 1 });
        let actual = grid.above(starting_grid_pos);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_grid_pos_above_none() {
        let grid = new_2x2_grid();
        let starting_grid_pos = GridPos { x: 0, y: 1 };
        let expected = None;
        let actual = grid.above(starting_grid_pos);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_grid_pos_above_right_some() {
        let grid = new_2x2_grid();
        let starting_grid_pos = GridPos { x: 0, y: 0 };
        let expected = Some(GridPos { x: 1, y: 1 });
        let actual = grid.above_right(starting_grid_pos);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_grid_pos_above_right_none() {
        let grid = new_2x2_grid();
        let starting_grid_pos = GridPos { x: 1, y: 1 };
        let expected = None;
        let actual = grid.above_right(starting_grid_pos);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_grid_pos_right_some() {
        let grid = new_2x2_grid();
        let starting_grid_pos = GridPos { x: 0, y: 0 };
        let expected = Some(GridPos { x: 1, y: 0 });
        let actual = grid.right(starting_grid_pos);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_grid_pos_right_none() {
        let grid = new_2x2_grid();
        let starting_grid_pos = GridPos { x: 1, y: 0 };
        let expected = None;
        let actual = grid.right(starting_grid_pos);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_grid_pos_below_right_some() {
        let grid = new_2x2_grid();
        let starting_grid_pos = GridPos { x: 0, y: 1 };
        let expected = Some(GridPos { x: 1, y: 0 });
        let actual = grid.below_right(starting_grid_pos);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_grid_pos_below_right_none() {
        let grid = new_2x2_grid();
        let starting_grid_pos = GridPos { x: 1, y: 0 };
        let expected = None;
        let actual = grid.below_right(starting_grid_pos);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_grid_pos_below_some() {
        let grid = new_2x2_grid();
        let starting_grid_pos = GridPos { x: 0, y: 1 };
        let expected = Some(GridPos { x: 0, y: 0 });
        let actual = grid.below(starting_grid_pos);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_grid_pos_below_none() {
        let grid = new_2x2_grid();
        let starting_grid_pos = GridPos { x: 0, y: 0 };
        let expected = None;
        let actual = grid.below(starting_grid_pos);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_edges_should_be_detected() {
        #[rustfmt::skip]
        let chars: CharGrid = vec![
            vec!['∧','/'],
            vec!['∨','\\'],
        ];
        let grid = Grid::new_from_chars(chars);
        let mut expected: Vec<(GridPos, GridPos)> = vec![
            (GridPos::new(0, 0), GridPos::new(0, 1)),
            (GridPos::new(0, 0), GridPos::new(1, 0)),
            (GridPos::new(0, 0), GridPos::new(1, 1)),
            (GridPos::new(1, 0), GridPos::new(0, 0)),
            (GridPos::new(1, 0), GridPos::new(0, 1)),
            (GridPos::new(1, 0), GridPos::new(1, 1)),
            (GridPos::new(0, 1), GridPos::new(0, 0)),
            (GridPos::new(0, 1), GridPos::new(1, 0)),
            (GridPos::new(0, 1), GridPos::new(1, 1)),
            (GridPos::new(1, 1), GridPos::new(0, 0)),
            (GridPos::new(1, 1), GridPos::new(1, 0)),
            (GridPos::new(1, 1), GridPos::new(0, 1)),
        ];
        expected.sort();

        let mut actual = grid.edges();
        actual.sort();

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_edges_should_not_be_detected() {
        #[rustfmt::skip]
        let chars: CharGrid = vec![
            vec!['∨','/'],
            vec!['∧','.'],
        ];
        let grid = Grid::new_from_chars(chars);
        let expected: Vec<(GridPos, GridPos)> = vec![];
        let actual = grid.edges();

        assert_eq!(expected, actual)
    }

    // '∧'
    // '∨'
    // '\\'
    // '/'

    #[test]
    fn test_recalculate_active_cells_all_active() {
        // env_logger::init();
        #[rustfmt::skip]
        let chars: CharGrid = vec![
            vec!['\\','/'],
            vec!['/','\\'],
        ];
        let mut grid = Grid::new_from_chars(chars);
        grid.recalculate_active_cells();

        #[rustfmt::skip]
        let expected_active: Bitmask = vec![
            vec![1,1],
            vec![1,1],
        ];
        let actual_active = grid.as_active_bitmask();

        assert_eq!(expected_active, actual_active);
    }

    #[test]
    fn test_recalculate_active_cells_all_inactive() {
        #[rustfmt::skip]
        let chars: CharGrid = vec![
            vec!['∨','/'],
            vec!['∧','\\'],
        ];
        let mut grid = Grid::new_from_chars(chars);
        grid.recalculate_active_cells();

        #[rustfmt::skip]
        let expected_active: Bitmask = vec![
            vec![0,0],
            vec![0,0],
        ];
        let actual_active = grid.as_active_bitmask();

        assert_eq!(expected_active, actual_active);
    }
}
