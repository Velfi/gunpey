use druid::im::Vector;

use crate::grid::get_pos_from_index;
use crate::{
    cell::Cell,
    grid::Grid,
    grid_pos::{gp, GridPos},
    line_fragment::{LineFragment, LineFragmentKind},
};
use std::collections::HashMap;

pub struct Adjacency {
    map: HashMap<GridPos, Vector<GridPos>>,
    width: usize,
}

type CornerNodes = (GridPos, GridPos);

impl Adjacency {
    pub fn from_grid(grid: &Grid) -> Self {
        let mut map: HashMap<GridPos, Vector<GridPos>> = HashMap::new();

        // for every filled cell, add an entry into the adjacency list
        // each cell is one "edge" between two "corners"
        for index in 0..grid.cells.len() {
            // map.insert(index.to_string(), Vec::new());
            match grid.cells.get(index).unwrap() {
                Cell::Filled(LineFragment { kind, .. }) => {
                    let (grid_pos_a, grid_pos_b) = index_to_corner_nodes(index, &kind, grid.width);

                    map.entry(grid_pos_a).or_default().push_back(grid_pos_b);
                    map.entry(grid_pos_b).or_default().push_back(grid_pos_a);
                }
                // empty cells have no edges
                Cell::Empty => (),
            }
        }

        Self {
            map,
            width: grid.width,
        }
    }

    pub fn corner_nodes_with_one_edge(&self) -> Vec<CornerNodes> {
        self.map
            .iter()
            .filter_map(|(k, v)| {
                // All nodes with only one connection that are not connected to the left or right sides
                // we use full width because these are corner nodes, not cell nodes
                if v.len() == 1 && k.x != 0 && k.x != self.width as isize {
                    Some((*k, v[0]))
                } else {
                    None
                }
            })
            .collect()
    }
}

pub fn index_to_corner_nodes(index: usize, kind: &LineFragmentKind, width: usize) -> CornerNodes {
    let cell_grid_pos = get_pos_from_index(index, width);

    (
        match kind {
            LineFragmentKind::Caret => cell_grid_pos,
            LineFragmentKind::InvertedCaret => cell_grid_pos + gp(0, 1),
            LineFragmentKind::LeftSlash => cell_grid_pos + gp(0, 1),
            LineFragmentKind::RightSlash => cell_grid_pos,
        },
        match kind {
            LineFragmentKind::Caret => cell_grid_pos + gp(1, 0),
            LineFragmentKind::InvertedCaret => cell_grid_pos + gp(1, 1),
            LineFragmentKind::LeftSlash => cell_grid_pos + gp(1, 0),
            LineFragmentKind::RightSlash => cell_grid_pos + gp(1, 1),
        },
    )
}

pub fn grid_pos_to_corner_nodes(grid_pos: GridPos, kind: &LineFragmentKind) -> CornerNodes {
    (
        match kind {
            LineFragmentKind::Caret => grid_pos,
            LineFragmentKind::InvertedCaret => grid_pos + gp(0, 1),
            LineFragmentKind::LeftSlash => grid_pos + gp(0, 1),
            LineFragmentKind::RightSlash => grid_pos,
        },
        match kind {
            LineFragmentKind::Caret => grid_pos + gp(1, 0),
            LineFragmentKind::InvertedCaret => grid_pos + gp(1, 1),
            LineFragmentKind::LeftSlash => grid_pos + gp(1, 0),
            LineFragmentKind::RightSlash => grid_pos + gp(1, 1),
        },
    )
}
