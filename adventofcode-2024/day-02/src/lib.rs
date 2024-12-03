use crate::LevelCheckResult::*;
use itertools::Itertools;
use miette::miette;
use nom::character::complete;
use nom::character::complete::{newline, space1};
use nom::multi::separated_list1;
use nom::IResult;

pub mod part1;
pub mod part2;

/// Report is a Vector of Levels
type Report = Vec<i32>;

fn nom_parser(input: &str) -> IResult<&str, Vec<Report>> {
    separated_list1(newline, separated_list1(space1, complete::i32))(input)
}

fn parse(input: &str) -> miette::Result<Vec<Report>> {
    // error needs to be mapped, because it contains &str in it that outlive the lifetime of the input &str
    let (_, reports) = nom_parser(input).map_err(|e| miette!("parse failed {}", e))?;

    Ok(reports)
}

#[derive(Eq, PartialOrd, PartialEq, Hash)]
enum LevelCheckResult {
    SafeIncreasing,
    SafeDecreasing,
    Unsafe,
}

fn validate_report(report: &Vec<i32>) -> bool {
    let analysis = report
        .into_iter()
        .tuple_windows()
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
            // dbg!(&new_report);
            // dbg!(&new_result);

            if new_result {
                //println!("found valid report by removing level {removed} at idx {problem_idx}");
                return true;
            }
        }
    }

    //println!("found no valid report by removing any level");

    false
}
