use crate::Error;
use omega_tile;
use omega_tile::{ts, SampleMode, WTileSet, WTileVariation};
use std::path::Path;
use ts::image::{DynamicImage, GenericImage, GenericImageView};

use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::{
    mpsc::{channel, Receiver, Sender, TryRecvError},
    Arc,
};
use std::thread;

#[derive(Clone)]
pub struct GenerateOptions {
    pub variation: WTileVariation,
    pub size: u32,
    pub seed: u64,
}

#[must_use]
pub struct Handle {
    inner: RefCell<Option<thread::JoinHandle<Result<PathBuf, Error>>>>,
    rx: Arc<Receiver<Arc<String>>>,
    last: RefCell<HandleResult<String>>,
}

#[derive(Clone)]
pub enum HandleResult<T> {
    Ok(T),
    Fail(Arc<Error>),
    Success(PathBuf),
}

impl Handle {
    pub fn get(&self) -> HandleResult<String> {
        let last = self.last.borrow().clone();

        if self.inner.borrow().is_none() {
            return last;
        }

        *self.last.borrow_mut() = match self.rx.try_recv() {
            Result::Ok(s) => HandleResult::Ok((*s).clone()),
            Result::Err(err) => match err {
                TryRecvError::Empty => return last,
                TryRecvError::Disconnected => {
                    let res = self.inner.borrow_mut().take().unwrap().join().unwrap();
                    match res {
                        Result::Err(it) => HandleResult::Fail(Arc::new(it)),
                        Result::Ok(it) => HandleResult::Success(it),
                    }
                }
            },
        };

        self.last.borrow().clone()
    }
}

struct DummyReport {
    tx: Arc<Sender<Arc<String>>>,
}

impl ts::GeneratorProgress for DummyProgress {
    fn update(&mut self, info: ts::ProgressUpdate<'_>) {
        let stage_percent = (info.stage.current as f64) / (info.stage.total as f64) * 100.0;
        let total_percent = (info.total.current as f64) / (info.total.total as f64) * 100.0;

        let s = format!(
            "{}: [{:02}/{:02}] [stage: {:05.2}, total: {:05.2}]",
            self.section.name,
            self.section.current,
            self.section.total,
            stage_percent,
            total_percent,
        );
        self.tx.send(Arc::new(s)).unwrap();
    }
}

struct DummyProgress {
    tx: Arc<Sender<Arc<String>>>,
    section: omega_tile::ReportSection,
}

impl omega_tile::Report for DummyReport {
    fn sub_progress_bar(
        &mut self,
        section: omega_tile::ReportSection,
    ) -> Box<dyn ts::GeneratorProgress> {
        Box::new(DummyProgress { section, tx: self.tx.clone() })
    }
}

pub fn generate(input: &Path, output: &Path, opt: &GenerateOptions) -> Handle {
    let input = input.to_owned();
    let output = output.to_owned();
    let opt = opt.clone();

    let (tx, rx) = channel();

    let t = thread::spawn(move || -> Result<std::path::PathBuf, Error> {
        let report = DummyReport { tx: Arc::new(tx) };

        // FIXME: Use cache in app ?
        let (tiles, _) = omega_tile::build(
            SampleMode::Split,
            &input.to_string_lossy(),
            opt.variation,
            report,
            None,
        )?;

        let tileset = build_tileset(&tiles)?;
        tileset.save(output.clone())?;
        Ok(output)
    });

    Handle {
        inner: RefCell::new(Some(t)),
        rx: Arc::new(rx),
        last: RefCell::new(HandleResult::Ok(String::new())),
    }
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
            Err(omega_tile::Error::SizeMismatch)?;
        }
    }

    Ok(combined)
}
