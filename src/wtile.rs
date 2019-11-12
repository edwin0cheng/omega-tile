use std::sync::Arc;
/// omega-tile
use texture_synthesis as ts;
use ts::image::DynamicImage;

#[derive(Clone, Debug, Copy)]
pub(crate) enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

#[derive(Clone, Debug)]
pub(crate) struct Edge(Direction, (usize, usize));

impl Edge {
    pub fn is_match(&self, other: &Edge) -> bool {
        self.1 == other.1
    }
}

#[derive(Clone)]
pub struct WTile {
    pub img: Arc<DynamicImage>,
    edges: [Edge; 4],
}

impl WTile {
    pub fn new(img: DynamicImage, a: usize, b: usize, c: usize, d: usize) -> WTile {
        use Direction::*;
        WTile {
            img: Arc::new(img),
            edges: [
                Edge(North, (a, b)),
                Edge(East, (b, d)),
                Edge(South, (c, d)),
                Edge(West, (a, c)),
            ],
        }
    }

    pub(crate) fn is_connectable(&self, dir: Direction, other: &WTile) -> bool {
        let e: &Edge = match dir {
            Direction::North => &other.edges[Direction::South as usize],
            Direction::East => &other.edges[Direction::West as usize],
            Direction::South => &other.edges[Direction::North as usize],
            Direction::West => &other.edges[Direction::East as usize],
        };

        let my_edge = &self.edges[dir as usize];
        e.is_match(&my_edge)
    }
}
