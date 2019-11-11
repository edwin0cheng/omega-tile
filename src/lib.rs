// References:
// Generating an ω-tile set for texture synthesis
// https://www.comp.nus.edu.sg/~tants/w-tile/index.html
// https://www.comp.nus.edu.sg/~tants/w-tile/w-tile.pdf
//
// Free texture for testing:
// https://unsplash.com/

mod atlas;
pub mod cache;
mod error;
mod report;
mod wtile;

use imageproc::drawing::draw_filled_circle_mut;
use std::path::Path;

use report::SimpleProgressReport;
use wtile::WTile;

pub use atlas::{build_atlas, Atlas};
pub use error::Error;
pub use texture_synthesis as ts;
use ts::image::{DynamicImage, GenericImage, GenericImageView, Rgba, Luma};

pub type WTileSet = Vec<WTile>;

struct WTileContext {
    pb: SimpleProgressReport,
}

impl WTileContext {
    fn build_samples<Q>(&mut self, path: Q) -> Result<Vec<DynamicImage>, Error>
    where
        Q: AsRef<Path>,
    {
        let mut result = vec![];

        let dim = {
            let img = ts::image::open(&path)
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "Not Found"))?;
            img.dimensions()
        };

        let build_sample = |id| -> Result<_, Error> {
            let key = format!(
                "{}+{}+{}+{}+samples",
                dim.0,
                dim.1,
                &path.as_ref().to_string_lossy(),
                id
            );
            if let Some(img) = cache::read_cache(&key) {
                Ok(img)
            } else {
                let texsynth = ts::Session::builder()
                    .add_example(&path)
                    .output_size(ts::Dims::new(dim.0, dim.1))
                    .seed(id)
                    .build()?;
                let generated =
                    texsynth.run(Some(Box::new(self.pb.new_sub_progress("build sample"))));
                let img = generated.into_image();
                cache::write_cache(&key, &img)?;
                Ok(img)
            }
        };

        for i in 1..=4 {
            result.push(build_sample(i)?);
        }

        Ok(result)
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

    fn build_mask(&self, (w, h): (u32, u32)) -> Result<DynamicImage, Error> {
        let mut res = DynamicImage::new_rgb8(w, h);
        for i in 0..w {
            for j in 0..h {
                res.put_pixel(i, j, Rgba([255u8, 255u8, 255u8, 255u8]));
            }
        }

        draw_filled_circle_mut(
            &mut res,
            ((w as i32) / 2, (h as i32) / 2),
            (w as i32) / 2,
            Rgba([0, 0, 0, 255u8]),
        );
        Ok(res)
    }

    fn build_tile<Q>(
        &mut self,
        merged: &DynamicImage,
        mask: &DynamicImage,
        base: Q,
    ) -> Result<DynamicImage, Error>
    where
        Q: AsRef<Path>,
    {
        let output_dim = mask.dimensions();

        let texsynth = ts::Session::builder()
            .add_examples(&[base])
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

        let generated = texsynth.run(Some(Box::new(self.pb.new_sub_progress("build tile"))));

        Ok(generated.into_image())
    }

    fn build_n_w_tiles_with_generator<F>(n_tiles: usize, mut gen: F) -> Result<Vec<WTile>, Error>
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
            4 => {
                // Figure 7(a)
                make_tile!(R, G, B, Y);
                make_tile!(G, B, Y, R);
                make_tile!(B, Y, R, G);
                make_tile!(Y, R, G, B);
            }
            16 => {
                // Figure 8(b)
                make_tile!(R, G, G, Y);
                make_tile!(R, B, G, R);
                make_tile!(R, B, B, Y);
                make_tile!(R, G, B, R);

                make_tile!(Y, G, G, R);
                make_tile!(Y, B, G, Y);
                make_tile!(Y, G, B, Y);
                make_tile!(Y, B, B, R);

                make_tile!(B, R, R, B);
                make_tile!(B, Y, R, G);
                make_tile!(B, R, Y, G);
                make_tile!(B, Y, Y, B);

                make_tile!(G, R, R, G);
                make_tile!(G, Y, R, B);
                make_tile!(G, R, Y, B);
                make_tile!(G, Y, Y, G);
            }
            _ => unimplemented!("Other tiles size is not supported right now"),
        }

        Ok(res)
    }

    fn build_n_w_tiles<Q>(
        &mut self,
        n_tiles: usize,
        samples: &[DynamicImage],
        mask: &DynamicImage,
        base: Q,
    ) -> Result<Vec<WTile>, Error>
    where
        Q: AsRef<Path> + std::fmt::Display,
    {
        Self::build_n_w_tiles_with_generator(n_tiles, |a, b, c, d| {
            let key = format!("{}+{}+{}+{}+{}+{}", n_tiles, base, a, b, c, d);
            let img = if let Some(img) = cache::read_cache(&key) {
                img
            } else {
                let merged = self.merge_samples(&samples, a, b, c, d)?;
                let img = self.build_tile(&merged, &mask, &base, &samples)?;
                cache::write_cache(&key, &img)?;
                img
            };

            Ok(img)
        })
    }

    fn build_test_tiles(
        &mut self,
        n_tiles: usize,
        samples: &[DynamicImage],
    ) -> Result<Vec<WTile>, Error> {
        Self::build_n_w_tiles_with_generator(n_tiles, |a, b, c, d| {
            let key = format!("{}+{}+{}+{}+{}+{}", n_tiles, "test", a, b, c, d);
            let img = if let Some(img) = cache::read_cache(&key) {
                img
            } else {
                let img = self.merge_samples(&samples, a, b, c, d)?;
                cache::write_cache(&key, &img)?;
                img
            };

            Ok(img)
        })
    }
}

pub fn build(base: &str, variation: usize) -> Result<WTileSet, Error> {
    let mut ctx = WTileContext {
        pb: SimpleProgressReport::new()
    };
    
    let samples = ctx.build_samples(&base)?;
    let mask = ctx.build_mask(samples[0].dimensions())?;

    ctx.build_n_w_tiles(variation, &samples, &mask, &base)
}

pub fn build_testset(variation: usize) -> Result<WTileSet, Error> {
    let mut ctx = WTileContext {
        pb: SimpleProgressReport::new(),
    };

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

        let mut r_img = DynamicImage::new_rgb8(256, 256);
        let mut g_img = DynamicImage::new_rgb8(256, 256);
        let mut b_img = DynamicImage::new_rgb8(256, 256);
        let mut y_img = DynamicImage::new_rgb8(256, 256);

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
