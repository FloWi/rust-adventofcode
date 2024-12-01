use anyhow::Result;
use clap::Parser;
use nom::character::complete::space0;
use nom::combinator::iterator;
use nom::{
    character::complete::{digit1, line_ending},
    combinator::map_res,
    sequence::{separated_pair, terminated},
    IResult,
};
use std::fmt::Debug;
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

// Single number parser
pub fn number(input: &str) -> IResult<&str, i32> {
    map_res(digit1, str::parse)(input)
}

// Parse a pair of numbers on one line
pub fn number_pair(input: &str) -> IResult<&str, (i32, i32)> {
    separated_pair(number, space0, number)(input)
}

pub fn streaming_parse<'a>(input: &'a str) -> impl Iterator<Item = (i32, i32)> + 'a {
    let mut it = iterator(input, terminated(number_pair, line_ending));
    std::iter::from_fn(move || (&mut it).next().map(|x| x))
}
