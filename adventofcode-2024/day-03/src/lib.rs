use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete;
use nom::character::complete::{anychar, char};
use nom::combinator::{map, value};
use nom::multi::many0;
use nom::sequence::{delimited, separated_pair};
use nom::IResult;
use std::cell::RefCell;
use std::convert::identity;
use std::rc::Rc;

pub mod part1;
pub mod part2;

#[derive(Clone, Debug, Copy)]
struct MultiplyOperation(i32, i32);

fn mul_op_parser(input: &str) -> IResult<&str, MultiplyOperation> {
    let (remaining, (n1, n2)) = delimited(
        tag("mul("),
        separated_pair(complete::i32, char(','), complete::i32),
        char(')'),
    )(input)?;
    Ok((remaining, MultiplyOperation(n1, n2)))
}

fn all_mul_ops_parser(input: &str) -> IResult<&str, Vec<MultiplyOperation>> {
    let mut parser = many0(alt((
        map(mul_op_parser, Some),
        // eat one char and discard it. But we need to return the same type as the mul_op_parser
        value(None, anychar),
    )));

    let (remaining, results): (&str, Vec<Option<MultiplyOperation>>) = parser(input)?;
    Ok((
        remaining,
        results.into_iter().filter_map(identity).collect_vec(),
    ))
}

#[derive(Debug)]
enum Instruction {
    SetEnabled,
    SetDisabled,
    Work(MultiplyOperation),
}

fn instruction_parser(input: &str) -> IResult<&str, Instruction> {
    alt((
        // Parse state changes
        map(tag("do()"), |_| Instruction::SetEnabled),
        map(tag("don't()"), |_| Instruction::SetDisabled),
        // Parse multiply operations
        map(mul_op_parser, Instruction::Work),
    ))(input)
}

fn parse_instructions(input: &str) -> IResult<&str, Vec<Instruction>> {
    let mut parser = many0(alt((
        map(instruction_parser, Some),
        // Consume one char and return None if no instruction matches
        map(anychar, |_| None),
    )));

    let (remaining, instructions) = parser(input)?;
    Ok((
        remaining,
        instructions.into_iter().filter_map(|x| x).collect(),
    ))
}
