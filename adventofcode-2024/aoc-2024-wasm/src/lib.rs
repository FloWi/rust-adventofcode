use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Clone)]
#[wasm_bindgen]
pub struct Solution {
    result: String,
    error: Option<String>,
}

#[wasm_bindgen]
impl Solution {
    #[wasm_bindgen(getter)]
    pub fn result(&self) -> String {
        self.result.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> Option<String> {
        self.error.clone()
    }
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
#[derive(Debug)]
pub enum Part {
    Part1 = 1,
    Part2 = 2,
}

#[wasm_bindgen]
pub fn solve_day(day: u8, part: Part, input: &str) -> Solution {
    match solve_day_internal(day, part, input) {
        Ok(result) => Solution {
            result,
            error: None,
        },
        Err(err) => Solution {
            result: String::new(),
            error: Some(err.to_string()),
        },
    }
}

fn solve_day_internal(day: u8, part: Part, input: &str) -> miette::Result<String> {
    let input = input.trim();

    match (day, part) {
        (01, Part::Part1) => day_01::part1::process(input),
        (01, Part::Part2) => day_01::part2::process(input),
        (02, Part::Part1) => day_02::part1::process(input),
        (02, Part::Part2) => day_02::part2::process(input),
        (03, Part::Part1) => day_03::part1::process(input),
        (03, Part::Part2) => day_03::part2::process(input),
        (04, Part::Part1) => day_04::part1::process(input),
        (04, Part::Part2) => day_04::part2::process(input),
        (05, Part::Part1) => day_05::part1::process(input),
        (05, Part::Part2) => day_05::part2::process(input),
        (06, Part::Part1) => day_06::part1::process(input),
        (06, Part::Part2) => day_06::part2::process(input),
        (07, Part::Part1) => day_07::part1::process(input),
        (07, Part::Part2) => day_07::part2::process(input),
        (08, Part::Part1) => day_08::part1::process(input),
        (08, Part::Part2) => day_08::part2::process(input),
        (09, Part::Part1) => day_09::part1::process(input),
        (09, Part::Part2) => day_09::part2::process(input),
        (10, Part::Part1) => day_10::part1::process(input),
        (10, Part::Part2) => day_10::part2::process(input),
        (11, Part::Part1) => day_11::part1::process(input),
        (11, Part::Part2) => day_11::part2::process(input),
        (12, Part::Part1) => day_12::part1::process(input),
        (12, Part::Part2) => day_12::part2::process(input),
        (13, Part::Part1) => day_13::part1::process(input),
        (13, Part::Part2) => day_13::part2::process(input),
        (14, Part::Part1) => day_14::part1::process(input),
        (14, Part::Part2) => day_14::part2::process(input),
        (15, Part::Part1) => day_15::part1::process(input),
        (15, Part::Part2) => day_15::part2::process(input),
        (16, Part::Part1) => day_16::part1::process(input),
        (16, Part::Part2) => day_16::part2::process(input),
        (17, Part::Part1) => day_17::part1::process(input),
        (17, Part::Part2) => day_17::part2::process(input),
        (18, Part::Part1) => day_18::part1::process(input),
        (18, Part::Part2) => day_18::part2::process(input),
        (19, Part::Part1) => day_19::part1::process(input),
        (19, Part::Part2) => day_19::part2::process(input),
        (20, Part::Part1) => day_20::part1::process(input),
        (20, Part::Part2) => day_20::part2::process(input),
        (21, Part::Part1) => day_21::part1::process(input),
        (21, Part::Part2) => day_21::part2::process(input),
        (22, Part::Part1) => day_22::part1::process(input),
        (22, Part::Part2) => day_22::part2::process(input),
        (23, Part::Part1) => day_23::part1::process(input),
        (23, Part::Part2) => day_23::part2::process(input),
        (24, Part::Part1) => day_24::part1::process(input),
        (24, Part::Part2) => day_24::part2::process(input),
        (25, Part::Part1) => day_25::part1::process(input),
        (day, part) => panic!("Day {day} Part {part:?} not included"),
    }
}