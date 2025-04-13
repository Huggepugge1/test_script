use crate::error::LexerError;
use crate::exitcode::ExitCode;
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

    #[clap(short = 'M', long)]
    pub disable_magic_warnings: bool,

    #[clap(short, long, default_value = "3")]
    pub max_size: u32,

    #[clap(short, long)]
    pub debug: bool,
}

pub fn run() {
    let args = Args::parse();

    if !args.file.exists() {
        LexerError::FileNotFound(&args.file).print();
        std::process::exit(ExitCode::SourceFileNotFound as i32);
    }

    match args.file.extension() {
        Some(ext) if ext == "tesc" => (),
        _ => {
            LexerError::FileExtensionNotTesc(&args.file).print();
            std::process::exit(ExitCode::FileExtentionNotTesc as i32);
        }
    }

    test::run(args);
}
