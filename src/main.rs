use crate::helpers::Args;
use anyhow::{bail, Result};
use clap::Parser;

mod aoc_2024;
mod helpers;

fn solve_day(args: &Args) -> Result<String> {
    let input = helpers::read_input(args)?;

    match (args.year, args.day, args.part) {
        (2024, 1, 1) => aoc_2024::day_01::part1(input.as_str()),
        (2024, 1, 2) => aoc_2024::day_01::part2(input.as_str()),
        // Add more cases as you solve more days
        _ => bail!(
            "Solution for year {} day {} part {} not implemented",
            args.year, args.day, args.part
        )
    }
}

fn main() {
    let args = Args::parse();

    println!(
        "Solving Advent of Code {} Day {} Part {}",
        args.year, args.day, args.part
    );

    if let Err(e) = solve_day(&args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
