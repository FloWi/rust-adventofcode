use itertools::Itertools;
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
use std::collections::{HashMap, VecDeque};
use std::fmt::{Display, Formatter};
use std::ops::{BitAnd, BitOr, BitXor, Not};
use tracing::info;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, (initial_signals, gates)): (&str, (HashMap<Signal, bool>, Vec<Gate>)) =
        parse(input).map_err(|e| miette!("parse failed {}", e))?;

    let final_signals = evaluate_dag(&initial_signals, gates);
    let binary: String = final_signals
        .into_iter()
        .filter(|(key, _)| key.0.starts_with("z"))
        .sorted_by_key(|(key, _)| key.0)
        .rev() // most significant bit first
        .map(|(_, value)| (value as u8).to_string())
        .collect();

    let result = u64::from_str_radix(&binary, 2).unwrap();
    Ok(result.to_string())
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
struct Signal<'a>(&'a str);

#[derive(Debug, Clone)]
enum Operator {
    XOR,
    OR,
    AND,
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Operator::XOR => "XOR",
            Operator::OR => "OR",
            Operator::AND => "AND",
        };
        f.write_str(label)
    }
}

#[derive(Debug)]
struct Gate<'a> {
    in_1: Signal<'a>,
    in_2: Signal<'a>,
    op: Operator,
    out: Signal<'a>,
}

impl Gate<'_> {
    pub(crate) fn eval(&self, in1: bool, in2: bool) -> bool {
        match self.op {
            Operator::XOR => in1.bitxor(in2),
            Operator::OR => in1.bitor(in2),
            Operator::AND => in1.bitand(in2),
        }
    }
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
    info!("rest: \n{input}");

    let (input, gates) = preceded(
        multispace1,
        separated_list1(
            line_ending,
            separated_pair(
                tuple((
                    alphanumeric1.map(|s: &str| Signal(s)),
                    alt((
                        value(Operator::AND, tag(" AND ")),
                        value(Operator::OR, tag(" OR ")),
                        value(Operator::XOR, tag(" XOR ")),
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

fn evaluate_dag<'a>(
    initial_signals: &'a HashMap<Signal<'a>, bool>,
    gates: Vec<Gate<'a>>,
) -> HashMap<Signal<'a>, bool> {
    let mut signals = initial_signals.clone();

    let mut open_list_gates: VecDeque<Gate> = gates.into();

    while open_list_gates.is_empty().not() {
        if let Some(pos) = open_list_gates
            .iter()
            .position(|g| signals.contains_key(&g.in_1) && signals.contains_key(&g.in_2))
        {
            let gate = open_list_gates.remove(pos).unwrap();
            let signal_in_1 = &gate.in_1;
            let signal_in_2 = &gate.in_2;
            let op = &gate.op;
            let in_1 = signals[signal_in_1];
            let in_2 = signals[signal_in_2];

            let out = gate.eval(in_1, in_2);
            signals.insert(gate.out.clone(), out);
            info!(
                "evaluated {} {} {} = {:?} ==> {} {} {} = {}",
                signal_in_1.0, op, signal_in_2.0, gate.out.0, in_1, op, in_2, out
            )
        }
    }

    signals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_small_example() -> miette::Result<()> {
        let input = r#"
x00: 1
x01: 1
x02: 1
y00: 0
y01: 1
y02: 0

x00 AND y00 -> z00
x01 XOR y01 -> z01
x02 OR y02 -> z02      "#
            .trim();
        assert_eq!("4", process(input)?);
        Ok(())
    }
    #[test]
    fn test_process_large_example() -> miette::Result<()> {
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
tnw OR pbm -> gnj
      "#
        .trim();
        assert_eq!("2024", process(input)?);
        Ok(())
    }
}
