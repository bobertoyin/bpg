use std::path::PathBuf;

use clap::Parser;
use num::rational::Ratio;

#[derive(Parser, Debug)]
pub struct Args {
    pub files: Vec<PathBuf>,
    #[clap(short, long, default_value_t = 400)]
    pub border: u32,
    #[clap(short, long)]
    pub ratio: Option<Ratio<u32>>,
}
