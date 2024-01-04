//! Command line arguments.
use std::path::PathBuf;

use clap::Parser;

/**
 * Command line arguments.
 */
#[derive(Parser, Debug)]
pub struct Args {
    /// Image file paths.
    pub files: Vec<PathBuf>,
    #[clap(short, long, default_value_t = 400)]
    /// Border size, in pixels.
    pub border: u32,
}
