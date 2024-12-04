use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete;
use nom::character::complete::{anychar, char};
use nom::combinator::map;
use nom::multi::{many1, many_till};
use nom::sequence::{delimited, separated_pair};
use nom::IResult;
use nom::Parser;

pub mod part1;
pub mod part2;

fn multiply_instruction_parser(input: &str) -> IResult<&str, Instruction> {
    let (remaining, (n1, n2)) = delimited(
        tag("mul("),
        separated_pair(complete::i32, char(','), complete::i32),
        char(')'),
    )(input)?;
    Ok((remaining, Instruction::MultiplyOperation(n1, n2)))
}

fn all_mul_ops_parser(input: &str) -> IResult<&str, Vec<Instruction>> {
    many1(many_till(anychar, multiply_instruction_parser).map(|(_discard, op)| op))(input)
}

#[derive(Debug)]
enum Instruction {
    SetEnabled,
    SetDisabled,
    MultiplyOperation(i32, i32),
}

fn instructions_parser(input: &str) -> IResult<&str, Instruction> {
    alt((
        // Parse state changes
        map(tag("do()"), |_| Instruction::SetEnabled),
        map(tag("don't()"), |_| Instruction::SetDisabled),
        // Parse multiply operations
        multiply_instruction_parser,
    ))(input)
}

fn parse_instructions(input: &str) -> IResult<&str, Vec<Instruction>> {
    many1(many_till(anychar, instructions_parser).map(|(_discard, instruction)| instruction))(input)
}
