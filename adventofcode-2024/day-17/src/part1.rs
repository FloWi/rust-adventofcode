use itertools::Itertools;
use miette::miette;
use nom::bytes::complete::tag;
use nom::character::complete;
use nom::character::complete::{anychar, char, multispace1};
use nom::combinator::all_consuming;
use nom::multi::separated_list1;
use nom::sequence::{preceded, separated_pair, tuple};
use nom::IResult;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, mut computer) = parse(input).map_err(|e| miette!("parse failed {}", e))?;

    dbg!(&computer);

    &computer.run();
    let output = computer
        .output
        .into_iter()
        .map(|num| num.to_string())
        .collect_vec()
        .join(",");

    Ok(output)
}

fn parse(input: &str) -> IResult<&str, Computer> {
    // TODO - check back after watching Chris' video on how to do the parsing of exactly 3 registers in a typesafe manner
    let (rest, (registers, program)) = all_consuming(separated_pair(
        separated_list1(
            multispace1,
            preceded(tuple((tag("Register "), anychar, tag(": "))), complete::u32),
        ),
        multispace1,
        preceded(tag("Program: "), separated_list1(char(','), complete::u32)),
    ))(input)?;

    if registers.len() != 3 {
        panic!("wrong number of registers")
    }

    Ok((
        rest,
        Computer {
            register_a: registers[0],
            register_b: registers[1],
            register_c: registers[2],
            program,
            instruction_pointer: 0,
            output: vec![],
        },
    ))
}

#[derive(Default, Clone, Debug)]
struct Computer {
    register_a: u32,
    register_b: u32,
    register_c: u32,
    program: Vec<u32>,
    instruction_pointer: u32,
    output: Vec<u32>,
}

impl Computer {
    pub(crate) fn run(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
Register A: 729
Register B: 0
Register C: 0

Program: 0,1,5,4,3,0
        "#
        .trim();
        assert_eq!("4,6,3,5,6,3,5,2,1,0", process(input)?);
        Ok(())
    }
}
