mod report;

use imageproc::drawing;
use omega_tile::{cache, ts, Atlas, Error, SampleMode, WTileSet, WTileVariation};
use rusttype::{FontCollection, Scale};
use std::path::Path;
use structopt::StructOpt;
use ts::image::{DynamicImage, GenericImage, GenericImageView, Rgba};

use report::SimpleProgressReport;

#[derive(Debug, StructOpt)]
#[structopt(name = "omega-tile", about = "ω-tile generator")]
enum Command {
    Clean,
    Build {
        #[structopt(short, long)]
        variation: WTileVariation,

        #[structopt(short, long)]
        combined: bool,

        #[structopt(short, long)]
        print_index: bool,

        input: String,
        size: u32,

        #[structopt(short, long, default_value = "100")]
        seed: u64,

        #[structopt(short, long)]
        number: bool,
    },
    TestSet {
        #[structopt(short, long)]
        variation: WTileVariation,

        #[structopt(short, long)]
        combined: bool,

        #[structopt(short, long)]
        print_index: bool,

        size: u32,

        #[structopt(short, long, default_value = "100")]
        seed: u64,

        #[structopt(short, long)]
        number: bool,
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

fn draw_number(
    image: &mut DynamicImage,
    n: u32,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) -> Result<(), Error> {
    let font = Vec::from(include_bytes!("DejaVuSans.ttf") as &[u8]);
    let font = FontCollection::from_bytes(font).unwrap().into_font().unwrap();

    let height = 12.4;
    let scale = Scale { x: height * 2.0, y: height };

    drawing::draw_text_mut(
        image,
        Rgba([0u8, 0u8, 255u8, 255u8]),
        x,
        y,
        scale,
        &font,
        &n.to_string(),
    );

    drawing::draw_text_mut(
        image,
        Rgba([0u8, 0u8, 255u8, 255u8]),
        x + w - (height.ceil() as u32) * (n.to_string().len() as u32),
        y + h - (height.ceil() as u32),
        scale,
        &font,
        &n.to_string(),
    );

    Ok(())
}

fn build_tileset(tiles: &WTileSet, with_number: bool) -> Result<DynamicImage, Error> {
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

        if with_number {
            draw_number(
                &mut combined,
                i as u32,
                (x as u32) * dim.0,
                (y as u32) * dim.1,
                dim.0,
                dim.1,
            )?;
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
        Command::Build { input, size, variation, combined, print_index, seed, number } => {
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

            let (tiles, samples) = omega_tile::build(
                SampleMode::Split,
                &input,
                variation,
                SimpleProgressReport::new(),
            )?;
            for (i, it) in samples.iter().enumerate() {
                let name = format!("out/{}_samples{}.png", output, i + 1);
                it.save(&name).map_err(|e| {
                    Error::General((
                        Box::new(e.into()),
                        format!("Fail to save samples to {}", name),
                    ))
                })?;
            }

            let combined_size = size;
            let atlas = omega_tile::build_atlas(&tiles, combined_size, seed);

            if combined {
                let combined = build_combine_img(&atlas)?;
                combined.save(format!(
                    "out/{}_combined_{}x{}_{}_{}.png",
                    output, combined_size, combined_size, variation, seed
                ))?;
            }

            let indices = atlas.build_indices();
            indices.save(format!(
                "out/{}_indices_{}x{}_{}_{}.bmp",
                output, combined_size, combined_size, variation, seed
            ))?;

            let tileset = build_tileset(&tiles, number)?;
            tileset.save(format!(
                "out/{}_tileset_{}x{}_{}_{}.png",
                output, combined_size, combined_size, variation, seed
            ))?;

            if print_index {
                println!("{}", atlas);
            }
        }
        Command::TestSet { size, combined, variation, print_index, seed, number } => {
            let output = "test_set";
            let tiles = omega_tile::build_testset(variation, SimpleProgressReport::new())?;

            let combined_size = size;
            let atlas = omega_tile::build_atlas(&tiles, combined_size, seed);

            if combined {
                let combined = build_combine_img(&atlas)?;
                combined.save(format!(
                    "out/{}_combined_{}x{}_{}_{}.png",
                    output, combined_size, combined_size, variation, seed
                ))?;
            }

            let indices = atlas.build_indices();
            indices.save(format!(
                "out/{}_indices_{}x{}_{}_{}.bmp",
                output, combined_size, combined_size, variation, seed
            ))?;

            let tileset = build_tileset(&tiles, number)?;
            tileset.save(format!(
                "out/{}_tileset_{}x{}_{}_{}.png",
                output, combined_size, combined_size, variation, seed
            ))?;

            if print_index {
                println!("{}", atlas);
            }
        }
    }
    Ok(())
}
