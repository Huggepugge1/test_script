use crate::test;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(index = 1)]
    pub file: PathBuf,

    #[clap(short, long, default_value = "2")]
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
