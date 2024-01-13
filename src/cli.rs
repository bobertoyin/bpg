//! Command line arguments.
use std::path::PathBuf;

use clap::Parser;
use num::rational::Ratio;

/**
 * Command line arguments.
 */
#[derive(Parser, Debug)]
#[command(author, version)]
#[command(about = "border-producing gizmo", long_about = None)]
pub struct Args {
    /// Image file paths.
    #[clap(required = true)]
    pub files: Vec<PathBuf>,
    #[clap(short, long, default_value_t = 400)]
    /// Border size, in pixels.
    pub border: u32,
    /// Force images to match this ratio.
    #[clap(short = 'r', long)]
    pub force_ratio: Option<Ratio<u32>>,
    /// Force orientation (this only applies when the ratio is forced: ratio-matching will always match the orientation).
    #[clap(short = 'o', long)]
    pub force_orientation: bool,
}
