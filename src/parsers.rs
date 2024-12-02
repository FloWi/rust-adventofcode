use nom::character::complete::{digit1, line_ending, space0, space1};
use nom::combinator::{iterator, map_res};
use nom::multi::separated_list0;
use nom::sequence::{separated_pair, terminated};
use nom::IResult;

// Single number parser
pub fn parse_number(input: &str) -> IResult<&str, i32> {
    map_res(digit1, str::parse)(input)
}

// Parse a pair of numbers on one line
pub fn parse_number_pair(input: &str) -> IResult<&str, (i32, i32)> {
    separated_pair(parse_number, space0, parse_number)(input)
}

pub fn parse_numbers(input: &str) -> IResult<&str, Vec<i32>> {
    let (input, _) = space0(input)?;
    separated_list0(space1, parse_number)(input)
}

pub fn streaming_parse_number_pair<'a>(input: &'a str) -> impl Iterator<Item = (i32, i32)> + 'a {
    let mut it = iterator(input, terminated(parse_number_pair, line_ending));
    std::iter::from_fn(move || (&mut it).next().map(|x| x))
}
