use anyhow::Result;
use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Year of the puzzle
    #[arg(long, value_parser = clap::value_parser!(u32).range(2015..))]
    pub(crate) year: u32,

    /// Day of the puzzle
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..=25))]
    pub(crate) day: u32,

    /// Part of the puzzle (1 or 2)
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..=2))]
    pub(crate) part: u32,

    /// Optional input file (if not provided, will use default input)
    #[arg(long)]
    pub(crate) input: Option<PathBuf>,
}

pub fn read_input(args: &Args) -> Result<String> {
    let input_path = match &args.input {
        None => {
            let day_input_path = format!("inputs/{}/day_{:02}.txt", args.year, args.day);
            &PathBuf::from_str(day_input_path.as_str())?
        }
        Some(custom_path) => custom_path,
    };
    println!(
        "{}: {}",
        if args.input.is_some() {
            "using custom input"
        } else {
            "input"
        },
        input_path.display()
    );

    let input = fs::read_to_string(input_path)?;
    Ok(input)
}
