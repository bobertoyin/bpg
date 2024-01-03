use std::{
    collections::HashSet,
    ffi::OsString,
    path::{Path, PathBuf},
};

use clap::Parser;
use image::{imageops::overlay, io::Reader, GenericImageView, ImageResult, Rgba, RgbaImage};
use num::rational::Ratio;
use once_cell::sync::Lazy;
use rayon::iter::{
    IntoParallelIterator, IntoParallelRefIterator, ParallelExtend, ParallelIterator,
};

use bpg::Args;

static COMMON_RATIOS: Lazy<Vec<Ratio<u32>>> = Lazy::new(|| {
    vec![
        Ratio::new(1, 1),
        Ratio::new(3, 2),
        Ratio::new(2, 3),
        Ratio::new(4, 3),
        Ratio::new(3, 4),
        Ratio::new(16, 9),
        Ratio::new(9, 16),
    ]
});

fn main() {
    let args = Args::parse();

    let mut files = HashSet::new();

    if !args.files.is_empty() {
        files.par_extend(args.files);
    }

    let results: Vec<(&Path, ImageResult<()>)> = files
        .par_iter()
        .map(|file| (file.as_path(), process_file(file.as_path(), args.border)))
        .collect();

    results
        .into_par_iter()
        .for_each(|(path, result)| report_result(path, &result));
}

fn report_result(path: &Path, result: &ImageResult<()>) {
    match result {
        Ok(_) => println!("✅ {}", path.display()),
        Err(e) => println!("❌ {}: {}", path.display(), e),
    };
}

fn process_file(file: &Path, border: u32) -> ImageResult<()> {
    let image = Reader::open(file)?.decode()?;
    let (w, h) = image.dimensions();
    let image_ratio = Ratio::new(w, h);
    let (best_ratio, _) = COMMON_RATIOS
        .par_iter()
        .map(|cr| {
            let check = cr * image_ratio.recip();
            let proximity = ((*check.numer() as f32 / *check.denom() as f32) - 1.0).abs();
            (*cr, proximity)
        })
        .reduce(
            || (Ratio::from_integer(u32::MAX), f32::MAX),
            |a, b| {
                if b.1 < a.1 {
                    b
                } else {
                    a
                }
            },
        );
    let (new_w, new_h) = match w > h {
        true => {
            let new_w = w + border;
            let new_h = new_w * best_ratio.denom() / best_ratio.numer();
            (new_w, new_h)
        }
        false => {
            let new_h = h + border;
            let new_w = new_h * best_ratio.numer() / best_ratio.denom();
            (new_w, new_h)
        }
    };
    let mut background = RgbaImage::from_pixel(new_w, new_h, Rgba([255, 255, 255, 0]));
    let w_offset = (new_w - w) / 2;
    let h_offset = (new_h - h) / 2;
    overlay(&mut background, &image, w_offset as i64, h_offset as i64);
    let mut new_path = PathBuf::new();
    if let Some(stem) = file.file_stem() {
        let mut new_stem = OsString::new();
        new_stem.push(stem);
        new_stem.push("_bordered");
        new_path.set_file_name(new_stem);
    }
    if let Some(extension) = file.extension() {
        new_path.set_extension(extension);
    }
    background.save(new_path)
}
