use crate::aoc_2024::day_02::LevelCheckResult::{SafeDecreasing, SafeIncreasing, Unsafe};
use crate::parsers::parse_numbers;
use anyhow::Result;
use itertools::*;
use nom::character::complete::line_ending;
use nom::multi::separated_list1;
use std::fmt::Debug;

#[derive(Eq, PartialOrd, PartialEq, Hash)]
enum LevelCheckResult {
    SafeIncreasing,
    SafeDecreasing,
    Unsafe,
}

fn validate_report(report: &Vec<i32>) -> bool {
    let analysis = report
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

fn validate_report_with_problem_dampener(report: &Vec<i32>) -> bool {
    let result = validate_report(&report);

    if result == true {
        //println("initial report is valid - no need to remove a level");
        return true;
    } else {
        for (problem_idx, _) in report.iter().enumerate() {
            let mut new_report = report.clone();
            let _removed = new_report.remove(problem_idx);
            let new_result = validate_report(&new_report);
            dbg!(&new_report);
            dbg!(&new_result);

            if new_result {
                //println!("found valid report by removing level {removed} at idx {problem_idx}");
                return true;
            }
        }
    }

    //println!("found no valid report by removing any level");

    false
}

/// Report is a Vector of Levels
type Report = Vec<i32>;

fn parse_day02_input(input: &str) -> Result<Vec<Vec<i32>>> {
    // error needs to be mapped, because it contains &str in it that outlive the lifetime of the input &str

    let (_, reports) = separated_list1(line_ending, parse_numbers)(input)
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    Ok(reports)
}

pub(crate) fn part1(input: String) -> Result<String> {
    let reports = parse_day02_input(input.as_str())?;

    let valid_report_count = reports.into_iter().filter(validate_report).count();

    Ok(format!("{valid_report_count}"))
}

pub(crate) fn part2(input: String) -> Result<String> {
    let reports = parse_day02_input(input.as_str())?;

    let valid_report_count = reports
        .into_iter()
        .filter(validate_report_with_problem_dampener)
        .count();

    Ok(format!("{valid_report_count}"))
}
