use miette::miette;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete;
use nom::character::complete::{alphanumeric1, line_ending};
use nom::character::streaming::multispace1;
use nom::combinator::value;
use nom::multi::separated_list1;
use nom::sequence::{preceded, separated_pair, tuple};
use nom::{IResult, Parser};
use std::collections::HashMap;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, (initial_signals, gates)) = parse(input).map_err(|e| miette!("parse failed {}", e))?;

    dbg!(initial_signals, gates);

    let result = 42;

    Ok(result.to_string())
}

#[derive(Eq, PartialEq, Hash, Debug)]
struct Signal<'a>(&'a str);

#[derive(Debug, Clone)]
enum Operator {
    Xor,
    Or,
    And,
}

#[derive(Debug)]
struct Gate<'a> {
    in_1: Signal<'a>,
    in_2: Signal<'a>,
    op: Operator,
    out: Signal<'a>,
}

fn parse(input: &str) -> IResult<&str, (HashMap<Signal, bool>, Vec<Gate>)> {
    let (input, initial_map) = separated_list1(
        line_ending,
        separated_pair(
            alphanumeric1.map(|s: &str| Signal(s)),
            tag(": "),
            complete::u8.map(|num| match num {
                1 => true,
                0 => false,
                _ => unreachable!("not a bool"),
            }),
        ),
    )(input)?;

    //now proceeding with shadowed input (rest)
    println!("rest: \n{input}");

    let (input, gates) = preceded(
        multispace1,
        separated_list1(
            line_ending,
            separated_pair(
                tuple((
                    alphanumeric1.map(|s: &str| Signal(s)),
                    alt((
                        value(Operator::And, tag(" AND ")),
                        value(Operator::Or, tag(" OR ")),
                        value(Operator::Xor, tag(" XOR ")),
                    )),
                    alphanumeric1.map(|s: &str| Signal(s)),
                )),
                tag(" -> "),
                alphanumeric1.map(|s: &str| Signal(s)),
            )
            .map(|((in_1, op, in_2), out)| Gate {
                in_1,
                in_2,
                op,
                out,
            }),
        ),
    )(input)?;

    Ok((input, (initial_map.into_iter().collect(), gates)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
x00: 1
x01: 0
x02: 1
x03: 1
x04: 0
y00: 1
y01: 1
y02: 1
y03: 1
y04: 1

ntg XOR fgs -> mjb
y02 OR x01 -> tnw
kwq OR kpj -> z05
x00 OR x03 -> fst
tgd XOR rvg -> z01
vdt OR tnw -> bfw
bfw AND frj -> z10
ffh OR nrd -> bqk
y00 AND y03 -> djm
y03 OR y00 -> psh
bqk OR frj -> z08
tnw OR fst -> frj
gnj AND tgd -> z11
bfw XOR mjb -> z00
x03 OR x00 -> vdt
gnj AND wpb -> z02
x04 AND y00 -> kjc
djm OR pbm -> qhw
nrd AND vdt -> hwm
kjc AND fst -> rvg
y04 OR y02 -> fgs
y01 AND x02 -> pbm
ntg OR kjc -> kwq
psh XOR fgs -> tgd
qhw XOR tgd -> z09
pbm OR djm -> kpj
x03 XOR y03 -> ffh
x00 XOR y04 -> ntg
bfw OR bqk -> z06
nrd XOR fgs -> wpb
frj XOR qhw -> z04
bqk OR frj -> z07
y03 OR x01 -> nrd
hwm AND bqk -> z03
tgd XOR rvg -> z12
tnw OR pbm -> gnj        "#
            .trim();
        assert_eq!("2024", process(input)?);
        Ok(())
    }
}
