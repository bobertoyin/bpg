use std::{collections::HashSet, path::Path};

use clap::Parser;
use image::ImageResult;
use rayon::iter::{
    IntoParallelIterator, IntoParallelRefIterator, ParallelExtend, ParallelIterator,
};

use bpg::{cli::Args, process::process_and_save_local};

fn main() {
    let args = Args::parse();

    let mut files = HashSet::new();

    if !args.files.is_empty() {
        files.par_extend(args.files);

        let results: Vec<(&Path, ImageResult<()>)> = files
            .par_iter()
            .map(|file| {
                (
                    file.as_path(),
                    process_and_save_local(
                        file.as_path(),
                        args.border,
                        args.force_ratio,
                        args.force_orientation,
                    ),
                )
            })
            .collect();

        results
            .into_par_iter()
            .for_each(|(path, result)| report_result(path, &result));
    }
}

fn report_result(path: &Path, result: &ImageResult<()>) {
    match result {
        Ok(_) => println!("✅ {}", path.display()),
        Err(e) => println!("❌ {}: {}", path.display(), e),
    };
}
