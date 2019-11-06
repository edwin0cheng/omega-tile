use omega_tile::{cache, ts, Atlas, Error, WTileSet};
use std::path::Path;
use structopt::StructOpt;
use ts::image::{DynamicImage, GenericImage, GenericImageView};

#[derive(Debug, StructOpt)]
#[structopt(name = "omega-tile", about = "ω-tile generator")]
enum Command {
    Clean,
    Build {
        #[structopt(short, long)]
        simple: bool,

        #[structopt(short, long)]
        combined: bool,

        #[structopt(short, long)]
        print_index: bool,

        input: String,
        size: u32,
    },
}

fn build_combine_img(atlas: &Atlas) -> Result<DynamicImage, Error> {
    let dim = atlas.tile_dimensions();
    let full_dim = atlas.dimensions();
    let mut combined = DynamicImage::new_rgb8(full_dim.0, full_dim.1);

    let n = atlas.size() as i32;
    for y in 0..n {
        for x in 0..n {
            let (_id, w) = atlas.get(x, y).expect("Altas is not completed");

            if !combined.copy_from(
                &w.img.view(0, 0, dim.0, dim.1),
                (x as u32) * dim.0,
                (y as u32) * dim.1,
            ) {
                Err(Error::SizeMismatch)?;
            }
        }
    }
    Ok(combined)
}

fn build_tileset(tiles: &WTileSet) -> Result<DynamicImage, Error> {
    fn nearest_sqrt(n: u32) -> u32 {
        let mut i = 0u32;
        while n > i * i {
            i += 1
        }
        i
    }

    let dim = tiles[0].img.dimensions();
    // find nearest square
    let n = nearest_sqrt(tiles.len() as u32);
    let mut combined = DynamicImage::new_rgb8(dim.0 * n, dim.1 * n);

    let iter = (0..n).flat_map(|y| (0..n).map(move |x| (x, y)));
    let iter = iter.take(tiles.len());

    for (i, (x, y)) in iter.enumerate() {
        if !combined.copy_from(
            &tiles[i].img.view(0, 0, dim.0, dim.1),
            (x as u32) * dim.0,
            (y as u32) * dim.1,
        ) {
            Err(Error::SizeMismatch)?;
        }
    }

    Ok(combined)
}

fn main() -> Result<(), Error> {
    let cmd = Command::from_args();

    match cmd {
        Command::Clean => {
            cache::clear_cache();
            println!("Image cache is clean.");
        }
        Command::Build {
            input,
            size,
            simple,
            combined,
            print_index,
        } => {
            let output = Path::new(&input)
                .file_stem()
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::NotFound, "Input file not found")
                })?
                .to_str()
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Input file name is not valid",
                    )
                })?;

            let variation = if simple { 4 } else { 16 };
            let tiles = omega_tile::build(&input, variation)?;

            for (i, t) in tiles.iter().enumerate() {
                t.img.save(format!("out/{}_final{}.png", output, i + 1))?;
            }

            let combined_size = size;
            let atlas = omega_tile::build_atlas(&tiles, combined_size);

            if combined {
                let combined = build_combine_img(&atlas)?;
                combined.save(format!(
                    "out/{}_combined_{}x{}.png",
                    output, combined_size, combined_size
                ))?;
            }

            let indices = atlas.build_indices();
            indices.save(format!(
                "out/{}_indices_{}x{}.bmp",
                output, combined_size, combined_size
            ))?;

            let tileset = build_tileset(&tiles)?;
            tileset.save(format!(
                "out/{}_tileset_{}x{}.png",
                output, combined_size, combined_size
            ))?;

            if print_index {
                println!("{}", atlas);
            }
        }
    }
    Ok(())
}
