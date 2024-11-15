use crate::test;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(index = 1)]
    pub file: PathBuf,

    #[clap(short = 'W', long)]
    pub disable_warnings: bool,

    #[clap(short = 'S', long)]
    pub disable_style_warnings: bool,

    #[clap(short, long, default_value = "3")]
    pub max_size: u32,
}

pub fn run() {
    let args = Args::parse();

    if args.file.extension().expect("File extension must be tesc") != "tesc" {
        panic!("File extension must be tesc");
    } else if !args.file.exists() {
        panic!("File not found");
    }

    test::run(args);
}
