use crate::adjacency::{adjacency_of_grid_positions, Adjacency};
use crate::cell::Cell;
use crate::grid_algorithms::index_to_corner_nodes;
use crate::grid_iterator_2d::{new_xy_iter, GridIterDirectionX, GridIterDirectionY};
use crate::grid_pos::gp;
use crate::{
    error::GunpeyLibError,
    grid_pos::GridPos,
    line_fragment::{LineFragment, LineFragmentKind},
};
use druid::{im::Vector, Data};
use log::{debug, trace};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;

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

    pub fn new_from_str(grid_str: &str) -> Self {
        let rows: Vec<_> = grid_str.trim().split("\n").map(str::trim).collect();

        let width = rows[0].len();
        let height = rows.len();
        assert!(width > 0, "width of new Grid must be greater than 0!");
        assert!(height > 1, "height of new Grid must be greater than 1!");

        let cells = rows
            .iter()
            .rev()
            .flat_map(|row| row.split("").filter(|&c| !c.is_empty()).map(Cell::from_str))
            .collect();

        debug!(
            "creating new grid from str with width={}, height={}",
            width, height
        );

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
            .rev()
            .flat_map(|row| row.iter().map(Cell::from_char))
            .collect();

        debug!(
            "creating new grid from chars with width={}, height={}",
            width, height
        );

        Self {
            width,
            height,
            cells,
        }
    }

    pub fn as_chars(&self) -> CharGrid {
        self.cell_rows_in_render_order()
            .into_iter()
            .map(|x| x.iter().map(Cell::to_char).collect())
            .collect()
    }

    pub fn as_active_bitmask(&self) -> Bitmask {
        // im::Vector doesn't support chunks in the way I'd expect
        let cells: Vec<_> = self.cells.iter().cloned().collect();
        cells
            .chunks_exact(self.width)
            .rev()
            .map(|x| x.iter().map(|cell| cell.is_active() as u8).collect())
            .collect()
    }

    pub fn cell_rows_in_render_order(&self) -> Vec<Vec<Cell>> {
        let cells: Vec<_> = self.cells.iter().cloned().collect();
        cells
            .chunks_exact(self.width)
            .rev()
            .map(|row| row.to_vec())
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
            self.get_index_from_pos(&cell_pos_a),
            self.get_index_from_pos(&cell_pos_b),
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

    pub fn set_cell(&mut self, grid_pos: &GridPos, cell: Cell) {
        if let Some(current_cell) = self
            .get_index_from_pos(grid_pos)
            .and_then(|i| self.cells.get_mut(i))
        {
            *current_cell = cell;
            self.recalculate_active_cells();
        }
    }

    pub fn get_cell_at_pos(&self, grid_pos: &GridPos) -> Option<&Cell> {
        self.get_index_from_pos(grid_pos)
            .and_then(|index| self.cells.get(index))
    }

    pub fn get_mut_cell_at_pos(&mut self, grid_pos: &GridPos) -> Option<&mut Cell> {
        if let Some(index) = self.get_index_from_pos(grid_pos) {
            self.cells.get_mut(index)
        } else {
            None
        }
    }

    pub fn get_index_from_pos(&self, GridPos { x, y }: &GridPos) -> Option<usize> {
        let i = x + self.width as isize * y;

        (0..self.cells.len() as isize)
            .contains(&i)
            .then(|| i as usize)
    }

    pub fn get_pos_from_index(&self, index: usize) -> GridPos {
        get_pos_from_index(index, self.width)
    }

    pub fn recalculate_active_cells_old(&mut self) {
        let edges = self.edges();

        trace!("recalculation active cells from edges\n{:?}", edges);

        let mut is_connected_to_left_edge: HashSet<GridPos> = (0..self.height)
            .map(|y| GridPos {
                x: 0,
                y: y as isize,
            })
            .filter(|gp| !self.is_cell_empty(gp))
            .collect();

        let mut i = 0;
        loop {
            trace!("loop #{}", i);
            i += 1;
            let mut change = false;

            for &connected_gp in is_connected_to_left_edge.clone().iter() {
                for (gp1, gp2) in edges.iter() {
                    if *gp1 == connected_gp {
                        if is_connected_to_left_edge.insert(*gp2) {
                            change = true;
                        }
                    } else if *gp2 == connected_gp && is_connected_to_left_edge.insert(*gp1) {
                        change = true;
                    }
                }
            }

            if !change {
                break;
            }
        }

        trace!("is_connected_to_left_edge loop exited");

        let mut is_connected_to_right_edge: HashSet<GridPos> = (0..self.height)
            .map(|y| GridPos {
                x: self.width as isize - 1,
                y: y as isize,
            })
            .filter(|gp| !self.is_cell_empty(gp))
            .collect();

        let mut i = 0;
        loop {
            trace!("loop #{}", i);
            i += 1;
            let mut change = false;

            for &connected_gp in is_connected_to_right_edge.clone().iter() {
                for (gp1, gp2) in edges.iter() {
                    if *gp1 == connected_gp {
                        if is_connected_to_right_edge.insert(*gp2) {
                            change = true;
                        }
                    } else if *gp2 == connected_gp && is_connected_to_right_edge.insert(*gp1) {
                        change = true;
                    }
                }
            }

            if !change {
                break;
            }
        }

        trace!("is_connected_to_right_edge loop exited");

        trace!("is_connected_to_left_edge\n{:?}", is_connected_to_left_edge);
        trace!(
            "is_connected_to_right_edge\n{:?}",
            is_connected_to_right_edge
        );

        let active_grid_positions: HashSet<_> = is_connected_to_left_edge
            .intersection(&is_connected_to_right_edge)
            .collect();

        // loop {
        //     let adjacency = crate::grid_algorithms::Adjacency::from_grid(self);
        //     let corner_nodes_with_one_edge = adjacency.corner_nodes_with_one_edge();

        //     if corner_nodes_with_one_edge.is_empty() {
        //         break;
        //     }

        //     for active_gp in active_grid_positions {
        //         let cell = self.get_cell_at_pos(active_gp).unwrap();
        //         let (cn_a, cn_b) = grid_pos_to_corner_nodes(active_gp, cell.);
        //     }
        // }

        // debug!(
        //     "corner_nodes_with_one_edge: {:?}",
        //     corner_nodes_with_one_edge
        // );

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

        trace!(
            "state of grid after recalculation of active cells:\n{}",
            self
        );
    }

    // Node - a corner in a grid of cells
    // Cell - a line between two corners or an empty space
    // Neighbor - a node connected to another node

    pub fn recalculate_active_cells(&mut self) {
        let mut nodes: HashSet<_> = new_xy_iter(
            self.width + 1,
            self.height + 1,
            GridIterDirectionX::LeftToRight,
            GridIterDirectionY::BottomToTop,
        )
        .map(|(x, y)| gp(x as isize, y as isize))
        .collect();

        'nodes: loop {
            for node in nodes.clone().iter() {
                let mut neighbor_count = 0;

                let other_cell = *node;
                if self.node_connects_across_cell(&other_cell, node, &nodes) {
                    neighbor_count += 1;
                }

                let other_cell = *node - gp(0, 1);
                if self.node_connects_across_cell(&other_cell, node, &nodes) {
                    neighbor_count += 1;
                }

                let other_cell = *node - gp(1, 1);
                if self.node_connects_across_cell(&other_cell, node, &nodes) {
                    neighbor_count += 1;
                }

                let other_cell = *node - gp(1, 0);
                if self.node_connects_across_cell(&other_cell, node, &nodes) {
                    neighbor_count += 1;
                }

                if neighbor_count < 2 {
                    nodes.remove(&node);
                    continue 'nodes;
                }
            }
            break 'nodes;
        }

        let mut cell_statuses: HashMap<GridPos, RefCell<CellStatus>> = HashMap::new();

        'cells: for (index, cell) in self.cells.iter_mut().enumerate() {
            let cell_pos = get_pos_from_index(index, self.width);
            if cell.is_empty() {
                cell_statuses.insert(
                    cell_pos,
                    RefCell::new(CellStatus {
                        is_connected_to_left_edge: false,
                        is_connected_to_right_edge: false,
                        is_part_of_a_chain: false,
                        cell_index: index,
                    }),
                );

                continue 'cells;
            }

            let corner_nodes = cell.corner_nodes(&cell_pos);
            for node in corner_nodes {
                if !nodes.contains(&node) {
                    cell.deactivate();
                    cell_statuses.insert(
                        cell_pos,
                        RefCell::new(CellStatus {
                            is_connected_to_left_edge: false,
                            is_connected_to_right_edge: false,
                            is_part_of_a_chain: false,
                            cell_index: index,
                        }),
                    );

                    continue 'cells;
                }
            }

            cell_statuses.insert(
                cell_pos,
                RefCell::new(CellStatus {
                    is_connected_to_left_edge: false,
                    is_connected_to_right_edge: false,
                    is_part_of_a_chain: true,
                    cell_index: index,
                }),
            );
        }

        debug!("cell_statuses={:#?}", cell_statuses);

        'edgeChecks: loop {
            let mut had_changes = false;
            for (cell_pos, cell_status) in cell_statuses.iter() {
                if !cell_status.borrow().is_part_of_a_chain {
                    continue;
                }

                let cell = self.get_cell_at_pos(cell_pos).unwrap();
                let neighbors = if cell_pos.x == 0 {
                    vec![
                        self.above_right(*cell_pos),
                        self.right(*cell_pos),
                        self.below_right(*cell_pos),
                    ]
                } else if cell_pos.x == (self.width - 1) as isize {
                    vec![
                        self.left(*cell_pos),
                        self.above_left(*cell_pos),
                        self.below_left(*cell_pos),
                    ]
                } else {
                    vec![
                        self.left(*cell_pos),
                        self.above_left(*cell_pos),
                        self.above(*cell_pos),
                        self.above_right(*cell_pos),
                        self.right(*cell_pos),
                        self.below_right(*cell_pos),
                        self.below(*cell_pos),
                        self.below_left(*cell_pos),
                    ]
                };

                let connected_neighbors: Vec<_> = neighbors
                    .into_iter()
                    .filter_map(|neighboring_pos| neighboring_pos)
                    .filter_map(|neighboring_pos| {
                        if let Some(neighboring_cell) = self.get_cell_at_pos(&neighboring_pos) {
                            Some((neighboring_pos, neighboring_cell))
                        } else {
                            None
                        }
                    })
                    .filter(|(neighboring_pos, neighboring_cell)| {
                        let adjacency = adjacency_of_grid_positions(*cell_pos, *neighboring_pos);

                        cell.is_connected_to(neighboring_cell, adjacency)
                    })
                    .collect();

                debug!("");

                // where "cell.connected_neighbors" is all neighbor cells that share a vertex,
                // UNLESS that vertex is on the left or right borders
                if !cell_status.borrow().is_connected_to_left_edge {
                    if connected_neighbors.iter().any(|(neighboring_pos, _)| {
                        cell_statuses
                            .get(neighboring_pos)
                            .map(|status| status.borrow().is_connected_to_left_edge)
                            .unwrap_or_default()
                    }) || cell_pos.x == 0
                    {
                        cell_status.borrow_mut().is_connected_to_left_edge = true;
                        had_changes = true;
                    }
                }

                if !cell_status.borrow().is_connected_to_right_edge {
                    if connected_neighbors.iter().any(|(neighboring_pos, _)| {
                        cell_statuses
                            .get(neighboring_pos)
                            .map(|status| status.borrow().is_connected_to_right_edge)
                            .unwrap_or_default()
                    }) || cell_pos.x == (self.width - 1) as isize
                    {
                        cell_status.borrow_mut().is_connected_to_right_edge = true;
                        had_changes = true;
                    }
                }
            }

            if !had_changes {
                break;
            }
        }

        for (cell_pos, cell_status) in cell_statuses.iter() {
            let cell = self.get_mut_cell_at_pos(cell_pos).unwrap();
            let cell_status = cell_status.borrow();

            if cell_status.is_connected_to_left_edge && cell_status.is_connected_to_right_edge {
                cell.activate();
            } else {
                cell.deactivate();
            }
        }
    }

    fn cells_to_nodes(&self) -> HashMap<usize, Vector<GridPos>> {
        let mut map: HashMap<usize, Vector<GridPos>> = HashMap::new();

        // for every filled cell, add an entry into the adjacency list
        // each cell is one "edge" between two "corners"
        for index in 0..self.cells.len() {
            // map.insert(index.to_string(), Vec::new());
            match self.cells.get(index).unwrap() {
                Cell::Filled(LineFragment { kind, .. }) => {
                    let (grid_pos_a, grid_pos_b) = index_to_corner_nodes(index, &kind, self.width);

                    let corners = map.entry(index).or_default();
                    corners.push_back(grid_pos_a);
                    corners.push_back(grid_pos_b);
                }
                // empty cells have no edges
                Cell::Empty => (),
            }
        }

        map
    }

    // #region hide

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
                    trace!(
                        "encountered filled cell at {} while building edge list, calculating edges",
                        gp1
                    );
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

        trace!("got possible edges, {:#?}", edges);

        edges.retain(|edge| self.cells_are_connecting(edge));
        trace!("after filtering edges for connections, {:#?}", edges);

        edges
    }

    fn cells_are_connecting(&self, (cell_a_pos, cell_b_pos): &(GridPos, GridPos)) -> bool {
        let adjacency = cell_a_pos.adjacency(cell_b_pos);

        trace!(
            "checking connection between {} and cell to the {} at {}",
            cell_a_pos,
            adjacency,
            cell_b_pos
        );

        match (
            self.get_index_from_pos(cell_a_pos),
            self.get_index_from_pos(cell_b_pos),
        ) {
            (Some(cell_a_index), Some(cell_b_index)) => {
                trace!("calculated indexes for both cells");

                let cell_a = self.cells.get(cell_a_index).unwrap();
                let cell_b = self.cells.get(cell_b_index).unwrap();

                trace!(
                    "checking connection between {:?} and cell to the {} at {:?}",
                    cell_a,
                    adjacency,
                    cell_b
                );

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

    fn node_connects_across_cell(
        &self,
        cell_pos: &GridPos,
        node_pos: &GridPos,
        nodes: &HashSet<GridPos>,
    ) -> bool {
        if !(0..self.width as isize).contains(&cell_pos.x) {
            return true;
        };
        if !(0..self.height as isize).contains(&cell_pos.y) {
            return false;
        };

        let cell = self.get_cell_at_pos(cell_pos).unwrap();
        if !cell.has_corner_node(cell_pos, node_pos) {
            return false;
        };
        for node in cell.corner_nodes(cell_pos) {
            if node == *node_pos {
                continue;
            }
            if nodes.contains(&node) {
                return true;
            };
        }

        false
    }

    pub fn pop_top_row(&mut self) -> Vector<Cell> {
        debug!("removing top x from grid");
        let y = self.height - 1;
        let start_of_last_row = y * self.width;
        let end_of_last_row = self.cells.len();
        let popped_row = self.cells.slice(start_of_last_row..end_of_last_row);

        popped_row
    }

    pub fn push_bottom_row(&mut self, mut new_row: Vector<Cell>) -> Result<(), GunpeyLibError> {
        if new_row.len() != self.width {
            return Err(GunpeyLibError::InvalidRowLength(new_row.len(), self.width));
        }

        debug!("pushing new x to bottom of grid");
        new_row.append(self.cells.clone());
        self.cells = new_row;

        self.recalculate_active_cells();

        Ok(())
    }

    pub fn is_cell_active(&self, cell_pos: &GridPos) -> bool {
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

    pub fn is_cell_empty(&self, cell_pos: &GridPos) -> bool {
        let cell_index = self
            .get_index_from_pos(cell_pos)
            .unwrap_or_else(|| panic!("bad grid pos {}, can't get cell::is_empty", cell_pos));
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
        if grid_pos.y == self.height as isize - 1 {
            None
        } else {
            Some(GridPos {
                y: grid_pos.y + 1,
                ..grid_pos
            })
        }
    }

    pub fn right(&self, grid_pos: GridPos) -> Option<GridPos> {
        if grid_pos.x == self.width as isize - 1 {
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

impl Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.cell_rows_in_render_order() {
            for cell in row {
                write!(f, "{}", cell)?;
            }
            writeln!(f, "")?;
        }

        Ok(())
    }
}

pub fn get_pos_from_index(index: usize, width: usize) -> GridPos {
    let x = (index % width) as isize;
    let y = (index / width) as isize;

    GridPos { x, y }
}

#[derive(Debug, Clone, Copy)]
struct CellStatus {
    pub is_connected_to_left_edge: bool,
    pub is_connected_to_right_edge: bool,
    pub is_part_of_a_chain: bool,
    pub cell_index: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use druid::im::vector;
    use pretty_assertions::assert_eq;

    fn new_2x2_grid() -> Grid {
        Grid::new_from_chars(vec![vec!['.', '.'], vec!['.', '.']])
    }

    fn bitmask_from_str(_bitmask_str: &str) -> Bitmask {
        todo!()
    }

    #[test]
    fn test_get_pos_from_index() {
        let width = 3;

        let expected = GridPos { x: 0, y: 0 };
        let actual = get_pos_from_index(0, width);

        assert_eq!(expected, actual);

        let expected = GridPos { x: 2, y: 0 };
        let actual = get_pos_from_index(2, width);

        assert_eq!(expected, actual);

        let expected = GridPos { x: 2, y: 1 };
        let actual = get_pos_from_index(5, width);

        assert_eq!(expected, actual);
    }

    #[test]
    fn testget_cell_at_pos() {
        let grid = Grid::new_from_str(
            r#"
                .c.
                r.i
            "#,
        );

        assert_eq!(".", grid.get_cell_at_pos(&gp(0, 1)).unwrap().to_str());
        assert_eq!("c", grid.get_cell_at_pos(&gp(1, 1)).unwrap().to_str());
        assert_eq!(".", grid.get_cell_at_pos(&gp(2, 1)).unwrap().to_str());
        assert_eq!("r", grid.get_cell_at_pos(&gp(0, 0)).unwrap().to_str());
        assert_eq!(".", grid.get_cell_at_pos(&gp(1, 0)).unwrap().to_str());
        assert_eq!("i", grid.get_cell_at_pos(&gp(2, 0)).unwrap().to_str());
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
    fn test_edges_should_be_detected_1() {
        #[rustfmt::skip]
        let chars: CharGrid = vec![
            vec!['/','/'],
            vec!['/','/'],
        ];
        let grid = Grid::new_from_chars(chars);
        let mut expected: Vec<(GridPos, GridPos)> = vec![
            (GridPos::new(0, 0), GridPos::new(1, 1)),
            (GridPos::new(1, 1), GridPos::new(0, 0)),
        ];
        expected.sort();

        let mut actual = grid.edges();
        actual.sort();

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_edges_should_be_detected_2() {
        #[rustfmt::skip]
        let chars: CharGrid = vec![
            vec!['∨','∨'],
            vec!['∧','∧'],
        ];
        let grid = Grid::new_from_chars(chars);
        let expected: Vec<(GridPos, GridPos)> = vec![
            (GridPos::new(0, 0), GridPos::new(1, 0)),
            (GridPos::new(1, 0), GridPos::new(0, 0)),
            (GridPos::new(0, 1), GridPos::new(1, 1)),
            (GridPos::new(1, 1), GridPos::new(0, 1)),
        ];
        let actual = grid.edges();

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

    // #endregion

    #[test]
    fn test_recalculate_active_cells_are_active_1() {
        #[rustfmt::skip]
        let chars: CharGrid = vec![
            vec!['.','∧','.'],
            vec!['/','.','∨'],
        ];
        #[rustfmt::skip]
        let expected_active: Bitmask = vec![
            vec![0, 0, 0],
            vec![0, 0, 0],
        ];
        let mut grid = Grid::new_from_chars(chars);

        // None should be active before recalculation
        let actual_active = grid.as_active_bitmask();
        assert_eq!(expected_active, actual_active);

        grid.recalculate_active_cells();

        #[rustfmt::skip]
        let expected_active: Bitmask = vec![
            vec![0, 1, 0],
            vec![1, 0, 1],
        ];

        // All should be active after recalculation
        let actual_active = grid.as_active_bitmask();
        assert_eq!(expected_active, actual_active);
    }

    #[test]
    fn test_recalculate_active_cells_are_active_2() {
        #[rustfmt::skip]
        let chars: CharGrid = vec![
            vec!['.','.','.','∧','.'],
            vec!['∧','∧','/','.','\\'],
            vec!['.','\\','\\','/','∨'],
            vec!['.','.','∨','\\','∧'],
            vec!['.','.','.','.','∨'],
        ];
        #[rustfmt::skip]
        let expected_active: Bitmask = vec![
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0],
        ];
        let mut grid = Grid::new_from_chars(chars);

        // None should be active before recalculation
        let actual_active = grid.as_active_bitmask();
        assert_eq!(expected_active, actual_active);

        grid.recalculate_active_cells();

        #[rustfmt::skip]
        let expected_active: Bitmask = vec![
            vec![0, 0, 0, 1, 0],
            vec![1, 1, 1, 0, 1],
            vec![0, 1, 1, 1, 1],
            vec![0, 0, 1, 1, 1],
            vec![0, 0, 0, 0, 1],
        ];

        // All should be active after recalculation
        let actual_active = grid.as_active_bitmask();
        assert_eq!(expected_active, actual_active);
    }

    #[test]
    fn test_recalculate_active_cells_are_active_3() {
        #[rustfmt::skip]
        let grid = r#"
            .rc
            ri.
        "#;
        #[rustfmt::skip]
        let expected_active: Bitmask = vec![
            vec![0, 0, 0],
            vec![0, 0, 0],
        ];
        let mut grid = Grid::new_from_str(grid);

        // None should be active before recalculation
        let actual_active = grid.as_active_bitmask();
        assert_eq!(expected_active, actual_active);

        grid.recalculate_active_cells();

        #[rustfmt::skip]
        let expected_active: Bitmask = vec![
            vec![0, 0, 1],
            vec![1, 1, 0],
        ];

        // Connected cells should be active after recalculation
        let actual_active = grid.as_active_bitmask();
        assert_eq!(expected_active, actual_active);
    }

    #[test]
    fn test_recalculate_active_cells_all_inactive() {
        #[rustfmt::skip]
        let chars: CharGrid = vec![
            vec!['/','.'],
            vec!['.','/'],
        ];
        #[rustfmt::skip]
        let expected_active: Bitmask = vec![
            vec![0,0],
            vec![0,0],
        ];

        let mut grid = Grid::new_from_chars(chars);

        // None should be active before recalculation
        let actual_active = grid.as_active_bitmask();
        assert_eq!(expected_active, actual_active);

        grid.recalculate_active_cells();

        // None should be active after recalculation either
        let actual_active = grid.as_active_bitmask();
        assert_eq!(expected_active, actual_active);
    }

    #[test]
    fn test_recalculate_active_cells_1() {
        let mut grid = Grid::new_from_str(
            r#"
        .....
        .....
        .....
        .....
        .....
        .....
        .....
        .....
        .crlr
        ic...
        "#,
        );

        let expected_active = bitmask_from_str(
            r#"
        00000
        00000
        00000
        00000
        00000
        00000
        00000
        00000
        00000
        01111
        10000
        "#,
        );

        // None should be active before recalculation
        let actual_active = grid.as_active_bitmask();
        assert_eq!(expected_active, actual_active);

        grid.recalculate_active_cells();

        // None should be active after recalculation either
        let actual_active = grid.as_active_bitmask();
        assert_eq!(expected_active, actual_active);
    }

    #[test]
    fn test_pop_top_row() {
        let mut grid = Grid::new_from_str(
            r#"
        .cc
        .l.
        ..l
        "#,
        );

        let expected_popped_row = vector![
            Cell::Empty,
            Cell::Filled(LineFragment::from_str("c")),
            Cell::Filled(LineFragment::from_str("c"))
        ];
        let actual_popped_row = grid.pop_top_row();

        assert_eq!(expected_popped_row, actual_popped_row);
    }
}
