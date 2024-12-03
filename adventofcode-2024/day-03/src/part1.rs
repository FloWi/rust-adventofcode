use itertools::Itertools;
use miette::miette;
use nom::bytes::complete::tag;
use nom::character::complete;
use nom::character::complete::{anychar, char};
use nom::combinator::value;
use nom::multi::many0;
use nom::sequence::{delimited, separated_pair};
use nom::IResult;
use std::convert::identity;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, operations) = all_mul_ops_parser(input)
        .map_err(|e| miette!("parse failed {}", e))?;

    // dbg!(&operations);

    let result: i32 = operations.iter().map(|MultiplyOperation(x, y )| x*y).sum();

    Ok(format!("{result}"))
}


#[derive(Clone, Debug)]
struct MultiplyOperation(i32, i32);

fn all_mul_ops_parser(input: &str) -> IResult<&str, Vec<MultiplyOperation>> {
    fn mul_op_parser(input: &str) -> IResult<&str, MultiplyOperation> {
        let (remaining, (n1, n2)) = delimited(
            tag("mul("),
            separated_pair(complete::i32, char(','), complete::i32),
            char(')'),
        )(input)?;
        Ok((remaining, MultiplyOperation(n1, n2)))
    }

    let mut parser = many0(
        nom::branch::alt(
            (
                nom::combinator::map(mul_op_parser, Some),
                // eat one char and discard it. But we need to return the same type as the mul_op_parser
                value(None, anychar)
            )
        ));

    let (remaining, results): (&str, Vec<Option<MultiplyOperation>>) = parser(input)?;
    Ok((remaining, results.into_iter().filter_map(identity).collect_vec()))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
xmul(2,4)%&mul[3,7]!@^do_not_mul(5,5)+mul(32,64]then(mul(11,8)mul(8,5))
        "#
            .trim();
        assert_eq!("161", process(input)?);
        Ok(())
    }
}
