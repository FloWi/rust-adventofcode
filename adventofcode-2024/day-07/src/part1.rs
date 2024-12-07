use miette::miette;
use nom::bytes::complete::tag;
use nom::character::complete;
use nom::character::complete::{char, line_ending};
use nom::multi::separated_list1;
use nom::sequence::separated_pair;
use nom::IResult;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, calibration_equations) = parse(input).map_err(|e| miette!("parse failed {}", e))?;

    dbg!(calibration_equations);
    unimplemented!()
}

#[derive(Debug)]
struct CalibrationEquation {
    test_value: i64,
    operands: Vec<i64>,
}

fn calibration_equation_parser(input: &str) -> IResult<&str, CalibrationEquation> {
    let (remaining, (test_value, operands)) = separated_pair(
        complete::i64,
        tag(": "),
        separated_list1(char(' '), complete::i64),
    )(input)?;

    Ok((
        remaining,
        CalibrationEquation {
            test_value,
            operands,
        },
    ))
}

fn parse(input: &str) -> IResult<&str, Vec<CalibrationEquation>> {
    separated_list1(line_ending, calibration_equation_parser)(input)
}

enum Operator {
    Add,
    Multiply,
}

impl Operator {
    pub(crate) fn perform(&self, p0: i64, p1: i64) -> i64 {
        match self {
            Operator::Add => p0 + p1,
            Operator::Multiply => p0 * p1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
190: 10 19
3267: 81 40 27
83: 17 5
156: 15 6
7290: 6 8 6 15
161011: 16 10 13
192: 17 8 14
21037: 9 7 18 13
292: 11 6 16 20
        "#
        .trim();
        assert_eq!("", process(input)?);
        Ok(())
    }
}
