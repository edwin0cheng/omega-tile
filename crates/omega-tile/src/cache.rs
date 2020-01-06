use crate::error::Error;
use hex;
use sha2::{Digest, Sha256};
use std::fs::{self, ReadDir};
use texture_synthesis as ts;
use ts::image::DynamicImage;

pub(crate) fn read_cache(key: &str) -> Option<DynamicImage> {
    let mut hasher = Sha256::new();
    hasher.input(key.as_bytes());
    let h = hex::encode(hasher.result());
    ts::image::open(format!("temp/{}.png", h)).ok()
}

pub(crate) fn write_cache(key: &str, img: &DynamicImage) -> Result<(), Error> {
    let mut hasher = Sha256::new();
    hasher.input(key.as_bytes());
    let h = hex::encode(hasher.result());
    img.save(format!("temp/{}.png", h))?;

    Ok(())
}

pub fn clear_cache() {
    let cache_dir = fs::read_dir("temp");
    delete_dir_contents(cache_dir);
}

fn delete_dir_contents(read_dir_res: Result<ReadDir, std::io::Error>) {
    if let Ok(dir) = read_dir_res {
        for entry in dir {
            if let Ok(entry) = entry {
                let path = entry.path();

                if path.is_dir() {
                    fs::remove_dir_all(path).expect("Failed to remove a dir");
                } else {
                    fs::remove_file(path).expect("Failed to remove a file");
                }
            };
        }
    };
}
