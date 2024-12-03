pub mod part1;
pub mod part2;

use miette::miette;
use nom::{
    character::complete::{self, newline, space1},
    combinator::{iterator, opt},
    sequence::{separated_pair, terminated},
    IResult,
};

pub fn parse(input: &str) -> IResult<&str, (Vec<i32>, Vec<i32>)> {
    let mut it = iterator(
        input,
        terminated(
            separated_pair(complete::i32, space1, complete::i32),
            opt(newline),
        ),
    );

    let parsed = it.collect::<(Vec<i32>, Vec<i32>)>();
    let res: IResult<_, _> = it.finish();

    res.map(|(input, _)| (input, parsed))
}
