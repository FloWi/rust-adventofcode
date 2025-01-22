pub mod testcases;

use crate::testcases::{read_all_testcases, Testcase};
use crate::Part::{Part1, Part2};
use chrono::{Duration, TimeDelta, Utc};
use leptos::attr::r#for;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct Solution {
    pub result: String,
    pub error: Option<String>,
    pub duration: TimeDelta,
}

pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Part {
    Part1 = 1,
    Part2 = 2,
}

impl TryFrom<u32> for Part {
    type Error = String;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Part1),
            2 => Ok(Part2),
            unknown => Err(format!("Unknown Part {unknown}")),
        }
    }
}

pub fn solve_day(day: u32, part: Part, input: &str, maybe_args: Option<String>) -> Solution {
    use chrono;
    let start = Utc::now();
    let result = solve_day_internal(day, part, input, maybe_args);
    let end = Utc::now();
    let duration = end.signed_duration_since(start);
    match result {
        Ok(result) => Solution { result, error: None, duration },
        Err(err) => Solution {
            result: String::new(),
            error: Some(err.to_string()),
            duration,
        },
    }
}

pub fn get_testcases() -> Vec<Testcase> {
    let testcases = read_all_testcases();
    testcases
}

fn solve_day_internal(day: u32, part: Part, input: &str, maybe_args: Option<String>) -> miette::Result<String> {
    let input = input.trim();

    match (day, part, maybe_args) {
        (01, Part::Part1, _) => day_01::part1::process(input),
        (01, Part::Part2, _) => day_01::part2::process(input),
        (02, Part::Part1, _) => day_02::part1::process(input),
        (02, Part::Part2, _) => day_02::part2::process(input),
        (03, Part::Part1, _) => day_03::part1::process(input),
        (03, Part::Part2, _) => day_03::part2::process(input),
        (04, Part::Part1, _) => day_04::part1::process(input),
        (04, Part::Part2, _) => day_04::part2::process(input),
        (05, Part::Part1, _) => day_05::part1::process(input),
        (05, Part::Part2, _) => day_05::part2::process(input),
        (06, Part::Part1, _) => day_06::part1::process(input),
        (06, Part::Part2, _) => day_06::part2::process(input),
        (07, Part::Part1, _) => day_07::part1::process(input),
        (07, Part::Part2, _) => day_07::part2::process(input),
        (08, Part::Part1, _) => day_08::part1::process(input),
        (08, Part::Part2, _) => day_08::part2::process(input),
        (09, Part::Part1, _) => day_09::part1::process(input),
        (09, Part::Part2, _) => day_09::part2::process(input),
        (10, Part::Part1, _) => day_10::part1::process(input),
        (10, Part::Part2, _) => day_10::part2::process(input),
        (11, Part::Part1, _) => day_11::part1::process(input),
        (11, Part::Part2, _) => day_11::part2::process(input),
        (12, Part::Part1, _) => day_12::part1::process(input),
        (12, Part::Part2, _) => day_12::part2::process(input),
        (13, Part::Part1, _) => day_13::part1::process(input),
        (13, Part::Part2, _) => day_13::part2::process(input),
        (14, Part::Part1, maybe_args) => day_14::part1::process_with_args(input, &maybe_args.unwrap_or("".to_string())),
        (14, Part::Part2, _) => day_14::part2::process(input),
        (15, Part::Part1, _) => day_15::part1::process(input),
        (15, Part::Part2, _) => day_15::part2::process(input),
        (16, Part::Part1, _) => day_16::part1::process(input),
        (16, Part::Part2, _) => day_16::part2::process(input),
        (17, Part::Part1, _) => day_17::part1::process(input),
        (17, Part::Part2, _) => day_17::part2::process(input),
        (18, Part::Part1, _) => day_18::part1::process(input),
        (18, Part::Part2, maybe_args) => day_18::part2::process_with_args(input, &maybe_args.unwrap_or("".to_string())),
        (19, Part::Part1, _) => day_19::part1::process(input),
        (19, Part::Part2, _) => day_19::part2::process(input),
        (20, Part::Part1, maybe_args) => day_20::part1::process_with_args(input, &maybe_args.unwrap_or("".to_string())),
        (20, Part::Part2, maybe_args) => day_20::part2::process_with_args(input, &maybe_args.unwrap_or("".to_string())),
        (21, Part::Part1, _) => day_21::part1::process(input),
        (21, Part::Part2, _) => day_21::part2::process(input),
        (22, Part::Part1, _) => day_22::part1::process(input),
        (22, Part::Part2, _) => day_22::part2::process(input),
        (23, Part::Part1, _) => day_23::part1::process(input),
        (23, Part::Part2, _) => day_23::part2::process(input),
        (24, Part::Part1, _) => day_24::part1::process(input),
        (24, Part::Part2, _) => day_24::part2::process(input),
        (25, Part::Part1, _) => day_25::part1::process(input),
        (day, part, _) => panic!("Day {day} Part {part:?} not included"),
    }
}
