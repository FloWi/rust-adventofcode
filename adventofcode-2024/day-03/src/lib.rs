use std::cell::RefCell;
use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete;
use nom::character::complete::{anychar, char};
use nom::combinator::{map, value};
use nom::multi::many0;
use nom::sequence::{delimited, separated_pair};
use nom::IResult;
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
    let mut parser = many0(
        alt(
            (
                map(mul_op_parser, Some),
                // eat one char and discard it. But we need to return the same type as the mul_op_parser
                value(None, anychar)
            )
        ));

    let (remaining, results): (&str, Vec<Option<MultiplyOperation>>) = parser(input)?;
    Ok((remaining, results.into_iter().filter_map(identity).collect_vec()))
}

fn check_enabled(context: Rc<RefCell<bool>>) -> impl FnMut(&str) -> IResult<&str, &str> {
    move |input| {
        let (remaining, maybe_state) = alt((
            map(tag("do()"), |_| Some(true)),
            map(tag("don't()"), |_| Some(false)),
        ))(input)?;

        if let Some(new_state) = maybe_state {
            *context.borrow_mut() = new_state;
        }
        Ok((remaining, ""))
    }
}

fn with_state<'a, F>(
    mut parser: F,
    context: Rc<RefCell<bool>>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Option<(MultiplyOperation, bool)>> + 'a
where
    F: FnMut(&'a str) -> IResult<&'a str, MultiplyOperation> + 'a,
{
    move |input| {
        parser(input)
            .map(|(remaining, op)| (remaining, Some((op, *context.borrow()))))
    }
}

fn all_mul_ops_parser_with_state(input: &str) -> IResult<&str, Vec<(MultiplyOperation, bool)>> {
    let is_enabled = Rc::new(RefCell::new(true));

    let mut parser = many0(
        alt(
            (
                value(None, check_enabled(is_enabled.clone())),
                with_state(mul_op_parser, is_enabled.clone()),
                // eat one char and discard it. But we need to return the same type as the mul_op_parser
                value(None, anychar)
            )
        ));

    let (remaining, results): (&str, Vec<Option<(MultiplyOperation, bool)>>) = parser(input)?;
    Ok((remaining, results.into_iter().filter_map(identity).collect_vec()))
}
