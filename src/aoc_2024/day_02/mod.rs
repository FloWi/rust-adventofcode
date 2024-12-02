use crate::aoc_2024::day_02::LevelCheckResult::{SafeDecreasing, SafeIncreasing, Unsafe};
use crate::parsers::parse_numbers;
use anyhow::Result;
use itertools::*;
use nom::character::complete::line_ending;
use nom::multi::separated_list0;
use std::fmt::Debug;

#[derive(Eq, PartialOrd, PartialEq, Hash)]
enum LevelCheckResult {
    SafeIncreasing,
    SafeDecreasing,
    Unsafe,
}

fn validate_level(level: &Vec<i32>) -> bool {
    let analysis = level
        .into_iter()
        .tuple_windows::<(_, _)>()
        .map(|(n1, n2)| {
            let diff = (n1 - n2).abs();
            if !(1..=3).contains(&diff) {
                Unsafe
            } else {
                if n1 > n2 {
                    SafeDecreasing
                } else if n1 < n2 {
                    SafeIncreasing
                } else {
                    Unsafe
                }
            }
        })
        .counts();

    let num_safe_increasing = *analysis.get(&SafeIncreasing).unwrap_or(&0);
    let num_safe_decreasing = *analysis.get(&SafeDecreasing).unwrap_or(&0);
    let num_unsafe = *analysis.get(&Unsafe).unwrap_or(&0);

    let no_unsafe = num_unsafe == 0;
    let only_increasing = num_safe_increasing > 0 && num_safe_decreasing == 0;
    let only_decreasing = num_safe_increasing == 0 && num_safe_decreasing > 0;
    let is_valid = (only_increasing || only_decreasing) && no_unsafe;
    is_valid
}

pub(crate) fn part1(input: String) -> Result<String> {
    // error needs to be mapped, because it contains &str in it that outlive the lifetime of the input &str
    let (_, levels) = separated_list0(line_ending, parse_numbers)(input.as_str())
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    let valid_report_count = levels.into_iter().filter(validate_level).count();

    Ok(format!("{valid_report_count}"))
}

pub(crate) fn part2(input: String) -> Result<String> {
    unimplemented!()
}
