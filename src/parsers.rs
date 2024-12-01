use nom::character::complete::{digit1, line_ending, space0};
use nom::combinator::{iterator, map_res};
use nom::sequence::{separated_pair, terminated};
use nom::IResult;

// Single number parser
pub fn number(input: &str) -> IResult<&str, i32> {
    map_res(digit1, str::parse)(input)
}

// Parse a pair of numbers on one line
pub fn number_pair(input: &str) -> IResult<&str, (i32, i32)> {
    separated_pair(number, space0, number)(input)
}

pub fn streaming_parse_number_pair<'a>(input: &'a str) -> impl Iterator<Item = (i32, i32)> + 'a {
    let mut it = iterator(input, terminated(number_pair, line_ending));
    std::iter::from_fn(move || (&mut it).next().map(|x| x))
}
