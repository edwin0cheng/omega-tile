// References:
// Generating an ω-tile set for texture synthesis
// https://www.comp.nus.edu.sg/~tants/w-tile/index.html
// https://www.comp.nus.edu.sg/~tants/w-tile/w-tile.pdf
//
// Free texture for testing:
// https://unsplash.com/

mod atlas;
mod cache;
mod error;
mod wtile;

use imageproc::drawing::draw_filled_circle_mut;
use std::path::Path;

use wtile::WTile;

pub use atlas::{build_atlas, Atlas};
pub use cache::Cache;
pub use error::Error;
pub use texture_synthesis as ts;

use ts::image::{DynamicImage, GenericImage, GenericImageView, Luma, Pixel, Rgba};

pub type WTileSet = Vec<WTile>;

pub trait Report {
    fn sub_progress_bar(&mut self, name: &str) -> Box<dyn ts::GeneratorProgress>;
}

#[derive(Debug, Copy, Clone)]
pub enum WTileVariation {
    /// 4 tiles version
    V4,
    /// 16 tiles version
    V16,
    /// 256 Full version
    Full,
}

impl std::str::FromStr for WTileVariation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "v4" => Ok(WTileVariation::V4),
            "v16" => Ok(WTileVariation::V16),
            "full" => Ok(WTileVariation::Full),
            _ => Err(Error::ParseError("Not a valid variation".into())),
        }
    }
}

impl std::fmt::Display for WTileVariation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let s = match self {
            WTileVariation::V4 => "v4",
            WTileVariation::V16 => "v16",
            WTileVariation::Full => "full",
        };

        write!(f, "{}", s)
    }
}

struct WTileContext {
    pb: Box<dyn Report>,
    cache: Option<Cache>,
}

impl WTileContext {
    fn build_samples<Q>(&mut self, mode: SampleMode, path: Q) -> Result<Vec<DynamicImage>, Error>
    where
        Q: AsRef<Path>,
    {
        match mode {
            SampleMode::Generate => {
                let dim = {
                    let img = ts::image::open(&path).map_err(|_| {
                        std::io::Error::new(std::io::ErrorKind::NotFound, "Not Found")
                    })?;
                    img.dimensions()
                };

                let mut build_sample = |id| -> Result<_, Error> {
                    let key = format!(
                        "{}+{}+{}+{}+samples",
                        dim.0,
                        dim.1,
                        &path.as_ref().to_string_lossy(),
                        id
                    );
                    if let Some(img) = self.cache.as_mut().and_then(|it| it.read_cache(&key)) {
                        Ok(img)
                    } else {
                        let texsynth = ts::Session::builder()
                            .add_example(&path)
                            .output_size(ts::Dims::new(dim.0, dim.1))
                            .seed(id)
                            .build()?;
                        let generated =
                            texsynth.run(Some(self.pb.sub_progress_bar("build sample")));
                        let img = generated.into_image();
                        if let Some(cache) = self.cache.as_mut() {
                            cache.write_cache(&key, &img)?;
                        }
                        Ok(img)
                    }
                };

                let mut result = vec![];
                for i in 1..=4 {
                    result.push(build_sample(i)?);
                }
                Ok(result)
            }
            SampleMode::Split => {
                let img = ts::image::open(&path)
                    .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "Not Found"))?;

                // Check if its squared
                let dims = img.dimensions();
                if !is_splitable(dims) {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Give texture size is invalide. (Must be square and even",
                    );
                }

                let half = dims.0 / 2;
                let mut result: Vec<DynamicImage> = vec![];

                result.push(DynamicImage::ImageRgba8(img.view(0, 0, half, half).to_image()));
                result.push(DynamicImage::ImageRgba8(img.view(0, half, half, half).to_image()));
                result.push(DynamicImage::ImageRgba8(img.view(half, 0, half, half).to_image()));
                result.push(DynamicImage::ImageRgba8(img.view(half, half, half, half).to_image()));

                return Ok(result);

                fn is_splitable(dims: (u32, u32)) -> bool {
                    if dims.0 == 0 || dims.0 != dims.1 {
                        return false;
                    }
                    dims.0 % 2 == 0
                }
            }
        }
    }

    fn merge_samples(
        &mut self,
        imgs: &[DynamicImage],
        a: usize,
        b: usize,
        c: usize,
        d: usize,
    ) -> Result<DynamicImage, Error> {
        let (w, h) = imgs[0].dimensions();
        let (w2, h2) = (w / 2, h / 2);
        let mut res = DynamicImage::new_rgb8(w, h);

        // *-----------*
        // |  A  |  B  |
        // *-----------*
        // |  C  |  D  |
        // *-----------*

        // A
        if !res.copy_from(&imgs[a].view(w2, h2, w2, h2), 0, 0) {
            Err(Error::SizeMismatch)?;
        }

        // B
        if !res.copy_from(&imgs[b].view(0, h2, w2, h2), w2, 0) {
            Err(Error::SizeMismatch)?;
        }

        // C
        if !res.copy_from(&imgs[c].view(w2, 0, w2, h2), 0, h2) {
            Err(Error::SizeMismatch)?;
        }

        // D
        if !res.copy_from(&imgs[d].view(0, 0, w2, h2), w2, h2) {
            Err(Error::SizeMismatch)?;
        }

        Ok(res)
    }

    /// Build a cross star like mask
    fn build_mask(&self, (w, h): (u32, u32)) -> Result<DynamicImage, Error> {
        let mut res = DynamicImage::new_rgb8(w, h);
        for i in 0..w {
            for j in 0..h {
                res.put_pixel(i, j, Rgba([0, 0, 0, 0]));
            }
        }

        let (w, h) = (w as i32, h as i32);

        draw_filled_circle_mut(&mut res, (0, 0), w / 2, Rgba([255u8, 255u8, 255u8, 255u8]));
        draw_filled_circle_mut(&mut res, (w, 0), w / 2, Rgba([255u8, 255u8, 255u8, 255u8]));
        draw_filled_circle_mut(&mut res, (0, h), w / 2, Rgba([255u8, 255u8, 255u8, 255u8]));
        draw_filled_circle_mut(&mut res, (w, h), w / 2, Rgba([255u8, 255u8, 255u8, 255u8]));

        Ok(res)
    }

    fn build_tile<Q>(
        &mut self,
        merged: &DynamicImage,
        mask: &DynamicImage,
        _base: Q,
        samples: &[DynamicImage],
    ) -> Result<DynamicImage, Error>
    where
        Q: AsRef<Path>,
    {
        let output_dim = mask.dimensions();

        let examples: Vec<_> = samples.iter().map(|it| it.clone()).collect();

        let texsynth = ts::Session::builder()
            .add_examples(examples.into_iter())
            .inpaint_example(
                mask.clone(),
                // This will prevent sampling from the imgs/2.jpg, note that
                // we *MUST* provide at least one example to source from!
                ts::Example::builder(merged.clone())
                    //  .set_sample_method(ts::SampleMethod::Ignore),
                    .set_sample_method(mask.clone()),
                ts::Dims::new(output_dim.0, output_dim.1),
            )
            .build()?;

        let generated = texsynth.run(Some(self.pb.sub_progress_bar("build tile")));

        Ok(generated.into_image())
    }

    fn build_n_w_tiles_with_generator<F>(
        n_tiles: WTileVariation,
        mut gen: F,
    ) -> Result<Vec<WTile>, Error>
    where
        F: FnMut(usize, usize, usize, usize) -> Result<DynamicImage, Error>,
    {
        macro_rules! rgby {
            (R) => {
                0
            };
            (G) => {
                1
            };
            (B) => {
                2
            };
            (Y) => {
                3
            };
        }

        let mut res = vec![];

        macro_rules! make_tile {
            ($a:ident, $b:ident, $c:ident, $d:ident) => {
                let img = gen(rgby!($a), rgby!($b), rgby!($c), rgby!($d))?;
                res.push(WTile::new(img, rgby!($a), rgby!($b), rgby!($c), rgby!($d)));
            };
        }

        match n_tiles {
            // 4
            WTileVariation::V4 => {
                // Figure 7(a)
                make_tile!(R, G, B, Y);
                make_tile!(G, B, Y, R);
                make_tile!(B, Y, R, G);
                make_tile!(Y, R, G, B);
            }
            WTileVariation::V16 => {
                // // Figure 8(b)
                // make_tile!(R, G, G, Y);
                // make_tile!(R, B, G, R);
                // make_tile!(R, B, B, Y);
                // make_tile!(R, G, B, R);

                // make_tile!(Y, G, G, R);
                // make_tile!(Y, B, G, Y);
                // make_tile!(Y, G, B, Y);
                // make_tile!(Y, B, B, R);

                // make_tile!(B, R, R, B);
                // make_tile!(B, Y, R, G);
                // make_tile!(B, R, Y, G);
                // make_tile!(B, Y, Y, B);

                // make_tile!(G, R, R, G);
                // make_tile!(G, Y, R, B);
                // make_tile!(G, R, Y, B);
                // make_tile!(G, Y, Y, G);

                // Figure 8(a)
                make_tile!(R, G, G, B);
                make_tile!(R, B, G, Y);
                make_tile!(R, G, B, Y);
                make_tile!(R, B, B, R);

                make_tile!(G, B, B, Y);
                make_tile!(G, Y, B, R);
                make_tile!(G, B, Y, R);
                make_tile!(G, Y, Y, G);

                make_tile!(B, Y, Y, R);
                make_tile!(B, R, Y, G);
                make_tile!(B, Y, R, G);
                make_tile!(B, R, R, B);

                make_tile!(Y, R, R, G);
                make_tile!(Y, G, R, B);
                make_tile!(Y, R, G, B);
                make_tile!(Y, G, G, Y);
            }

            WTileVariation::Full => {
                for a in 0..4 {
                    for b in 0..4 {
                        for c in 0..4 {
                            for d in 0..4 {
                                let img = gen(a, b, c, d)?;
                                res.push(WTile::new(img, a, b, c, d));
                            }
                        }
                    }
                }
            }
        }

        Ok(res)
    }

    fn build_n_w_tiles<Q>(
        &mut self,
        n_tiles: WTileVariation,
        samples: &[DynamicImage],
        mask: &DynamicImage,
        base: Q,
    ) -> Result<Vec<WTile>, Error>
    where
        Q: AsRef<Path> + std::fmt::Display,
    {
        Self::build_n_w_tiles_with_generator(n_tiles, |a, b, c, d| {
            let key = format!("{}+{}+{}+{}+{}+{}", n_tiles, base, a, b, c, d);
            let img = if let Some(img) = self.cache.as_mut().and_then(|it| it.read_cache(&key)) {
                img
            } else {
                let merged = self.merge_samples(&samples, a, b, c, d)?;
                let img = self.build_tile(&merged, &mask, &base, &samples)?;
                if let Some(cache) = self.cache.as_mut() {
                    cache.write_cache(&key, &img)?;
                }
                img
            };

            Ok(img)
        })
    }

    fn build_test_tiles(
        &mut self,
        n_tiles: WTileVariation,
        samples: &[DynamicImage],
    ) -> Result<Vec<WTile>, Error> {
        Self::build_n_w_tiles_with_generator(n_tiles, |a, b, c, d| {
            let key = format!("{}+{}+{}+{}+{}+{}", n_tiles, "test", a, b, c, d);
            let img = if let Some(img) = self.cache.as_mut().and_then(|it| it.read_cache(&key)) {
                img
            } else {
                let img = self.merge_samples(&samples, a, b, c, d)?;
                if let Some(cache) = self.cache.as_mut() {
                    cache.write_cache(&key, &img)?;
                }
                img
            };

            Ok(img)
        })
    }
}

pub enum SampleMode {
    Generate,
    Split,
}

pub fn build(
    mode: SampleMode,
    base: &str,
    variation: WTileVariation,
    report: impl Report + 'static,
    cache: Option<Cache>,
) -> Result<(WTileSet, Vec<DynamicImage>), Error> {
    let mut ctx = WTileContext { pb: Box::new(report), cache };

    let samples = ctx
        .build_samples(mode, &base)
        .map_err(|e| Error::General((Box::new(e), "Fail to build samples".to_string())))?;

    let mask = ctx.build_mask(samples[0].dimensions())?;

    Ok((ctx.build_n_w_tiles(variation, &samples, &mask, &base)?, samples))
}

pub fn build_testset(
    variation: WTileVariation,
    report: impl Report + 'static,
    cache: Option<Cache>,
) -> Result<WTileSet, Error> {
    let mut ctx = WTileContext { pb: Box::new(report), cache };

    let samples = {
        let mut samples: Vec<DynamicImage> = Vec::new();

        fn fill(img: &mut DynamicImage, color: Rgba<u8>) {
            let dim = img.dimensions();
            for y in 0..dim.1 {
                for x in 0..dim.0 {
                    img.put_pixel(x, y, color);
                }
            }
        }

        let mut r_img = DynamicImage::new_rgb8(128, 128);
        let mut g_img = DynamicImage::new_rgb8(128, 128);
        let mut b_img = DynamicImage::new_rgb8(128, 128);
        let mut y_img = DynamicImage::new_rgb8(128, 128);

        fill(&mut r_img, Rgba::from_channels(255, 0, 0, 255));
        fill(&mut g_img, Rgba::from_channels(0, 255, 0, 255));
        fill(&mut b_img, Rgba::from_channels(0, 0, 255, 255));
        fill(&mut y_img, Rgba::from_channels(128, 128, 128, 255));

        samples.push(r_img);
        samples.push(g_img);
        samples.push(b_img);
        samples.push(y_img);

        samples
    };

    ctx.build_test_tiles(variation, &samples)
}
