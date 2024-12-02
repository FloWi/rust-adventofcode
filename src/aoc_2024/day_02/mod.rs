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
            if !(1..3).contains(&diff) {
                LevelCheckResult::Unsafe
            } else {
                if n1 > n2 {
                    LevelCheckResult::SafeDecreasing
                } else {
                    LevelCheckResult::SafeIncreasing
                }
            }
        })
        .counts();

    analysis.len() == 1 && analysis.get(&LevelCheckResult::Unsafe).is_none()
}

pub(crate) fn part1(input: &str) -> Result<String> {
    let (_0, levels) = separated_list0(line_ending, parse_numbers)(input)?;

    let valid_report_count = levels.into_iter().filter(validate_level).count();

    Ok(format!("{valid_report_count}"))
}

pub(crate) fn part2(input: &str) -> Result<String> {
    unimplemented!()
}
