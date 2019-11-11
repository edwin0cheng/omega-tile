use crate::{wtile, WTile};
use crate::{DynamicImage, GenericImageView, Luma};
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use std::collections::HashMap;
use std::fmt;

fn fit(x: i32, y: i32, w: &WTile, atlas: &HashMap<(i32, i32), (usize, WTile)>) -> bool {
    macro_rules! check {
        ($dir:ident, $dx:literal, $dy:literal) => {
            if let Some(other) = atlas.get(&(x + $dx, y + $dy)) {
                if !w.is_connectable(wtile::Direction::$dir, &other.1) {
                    return false;
                }
            }
        };
    }

    check!(North, 0, -1);
    check!(South, 0, 1);
    check!(East, 1, 0);
    check!(West, -1, 0);
    true
}

pub struct Atlas {
    data: HashMap<(i32, i32), (usize, WTile)>,
    n: u32,
    tile_dimensions: (u32, u32),
}

impl Atlas {
    pub fn tile_dimensions(&self) -> (u32, u32) {
        self.tile_dimensions
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (
            self.tile_dimensions.0 * self.n,
            self.tile_dimensions.1 * self.n,
        )
    }

    pub fn size(&self) -> u32 {
        self.n
    }

    pub fn get(&self, x: i32, y: i32) -> Option<(usize, WTile)> {
        self.data.get(&(x, y)).cloned()
    }

    pub fn build_indices(&self) -> DynamicImage {
        let mut res = DynamicImage::new_luma8(self.n, self.n);
        let img = res.as_mut_luma8().unwrap();

        for y in 0..self.n {
            for x in 0..self.n {
                let (id, _) = self
                    .get(x as i32, y as i32)
                    .expect("Altas is not completed");

                img.put_pixel(x, y, Luma([id as u8]));
            }
        }

        res
    }
}

impl fmt::Display for Atlas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.n {
            for x in 0..self.n {
                let (id, _) = self
                    .get(x as i32, y as i32)
                    .expect("Altas is not completed");

                if x == (self.n - 1) && y != (self.n - 1) {
                    writeln!(f, "{:02}", id)?;
                } else {
                    write!(f, "{:02} ", id)?;
                }
            }
        }

        Ok(())
    }
}

pub fn build_atlas(tiles: &Vec<WTile>, n: u32) -> Atlas {
    let mut atlas: HashMap<(i32, i32), (usize, WTile)> = HashMap::new();
    let mut rng = StdRng::seed_from_u64(100);
    let id_tiles: Vec<(usize, WTile)> = tiles.into_iter().cloned().enumerate().collect();

    let mut shuffle = || {
        let mut res: Vec<(usize, WTile)> = id_tiles.clone();
        res.shuffle(&mut rng);
        res
    };

    // Simple order
    // let iter = (0..n).flat_map(|y| (0..n).map(move |x| (x, y)));

    // first row and left most order
    let first_row = (0..n).map(|x| (0, x));
    let first_col = (1..n).map(|y| (y, 0));
    let inner_iter = (1..n).flat_map(|y| (1..n).map(move |x| (x, y)));
    let iter = first_row.chain(first_col).chain(inner_iter);

    // Generate a combined image
    // let dir = (usize,usize);

    for (x, y) in iter {
        let mut list = shuffle();
        let mut success = false;

        while let Some(cur) = list.pop() {
            if fit(x as i32, y as i32, &cur.1, &atlas) {
                atlas.insert((x as i32, y as i32), cur);
                success = true;
                break;
            }
        }

        if !success {
            panic!("Generate combined image fail!");
        }
    }

    Atlas {
        data: atlas,
        n,
        tile_dimensions: tiles[0].img.dimensions(),
    }
}
