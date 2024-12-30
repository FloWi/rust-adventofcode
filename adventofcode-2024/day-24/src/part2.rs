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
use petgraph::dot::Dot;
use petgraph::prelude::DiGraphMap;
use std::collections::{HashMap, VecDeque};
use std::convert::identity;
use std::fmt::{Display, Formatter};
use std::ops::{BitAnd, BitOr, BitXor, Not, RangeInclusive};
use tracing::{debug, info};
use tracing_subscriber::fmt::format;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, (initial_signals, gates)): (&str, (HashMap<&str, bool>, Vec<Gate>)) =
        parse(input).map_err(|e| miette!("parse failed {}", e))?;

    // Task is to fix the gates to perform the addition operation correctly.
    // x + y = z

    let mut gates = gates;

    // // causes a fix in 2^10 and 2^11
    //swap(&mut gates, 44, 78);
    //
    // // causes a fix in 2^14 and 2^15
    // swap(&mut gates, 53, 140);
    //
    // // causes a fix in 2^26 and 2^27
    // // swap(&mut gates, 20, 156);
    // swap(&mut gates, 60, 206);
    // // swap(&mut gates, 110, 182);

    // 2^34; x: 17179869183; y: 1; is_broken: true; z_decimal_expected: 17179869184; z_decimal_actual: 34359738368; z_binary_actual: 0000000000100000000000000000000000000000000000; z_binary_expected: 0000000000010000000000000000000000000000000000
    // found 5 swap-combinations that improve the result
    // num_diffs: 0; diffs: []; idx_1: 19; idx_2: 45; z_binary: 0000000000010000000000000000000000000000000000; z_decimal: 17179869184
    // num_diffs: 0; diffs: []; idx_1: 45; idx_2: 58; z_binary: 0000000000010000000000000000000000000000000000; z_decimal: 17179869184
    // num_diffs: 0; diffs: []; idx_1: 45; idx_2: 99; z_binary: 0000000000010000000000000000000000000000000000; z_decimal: 17179869184
    // num_diffs: 0; diffs: []; idx_1: 45; idx_2: 111; z_binary: 0000000000010000000000000000000000000000000000; z_decimal: 17179869184
    // num_diffs: 0; diffs: []; idx_1: 45; idx_2: 192; z_binary: 0000000000010000000000000000000000000000000000; z_decimal: 17179869184

    // causes a fix in last entry 2^34
    // swap(&mut gates, 19, 45);
    // swap(&mut gates, 45, 58);
    // swap(&mut gates, 45, 99);
    // swap(&mut gates, 45, 111);
    // swap(&mut gates, 45, 192);

    // idx_1: 64;  idx_2: 78;
    // idx_1: 78;  idx_2: 131;
    //
    // let wires_affected_by_swaps = manual_swaps
    //     .iter()
    //     .flat_map(|(a, b)| {
    //         let gate_a: &Gate = &gates[*a];
    //         let gate_b: &Gate = &gates[*b];
    //         vec![gate_a.out.to_string(), gate_b.out.to_string()]
    //     })
    //     .sorted()
    //     .join(",");
    //
    // println!("wires_affected_by_swaps: {wires_affected_by_swaps}");

    let final_signals = evaluate_dag(&initial_signals, &gates);

    let (x_binary, x_decimal) =
        read_binary_number_from_register_starting_with_char("x", &final_signals);
    let (y_binary, y_decimal) =
        read_binary_number_from_register_starting_with_char("y", &final_signals);
    let (z_binary, z_decimal_actual) =
        read_binary_number_from_register_starting_with_char("z", &final_signals);

    let z_decimal_expected = x_decimal + y_decimal;
    let z_binary_expected = format!("{:b}", z_decimal_expected);

    println!();
    println!();
    println!("         x_decimal: {x_decimal}");
    println!("         y_decimal: {y_decimal}");
    println!("z_decimal_expected: {z_decimal_expected}");
    println!("  z_decimal_actual: {z_decimal_actual}");

    println!("          x_binary: {x_binary}");
    println!("          y_binary: {y_binary}");
    println!("          z_binary: {z_binary}");
    println!(" z_binary_expected: {z_binary_expected}");

    let is_match = z_decimal_expected == z_decimal_actual;
    println!("did I fix it? {is_match}");

    // let broken_ranges: Vec<RangeInclusive<usize>> =
    //     find_broken_ranges_in_bit_string(&z_binary, &z_binary_expected);
    //
    // println!("broken_ranges: {:?}", broken_ranges);

    //render_graph(&gates, &final_signals);

    //find_swaps(&gates, &initial_signals, &z_binary_expected, &broken_ranges);
    check_basic_functionality(&gates, &initial_signals, x_decimal, y_decimal);

    Ok(z_decimal_actual.to_string())
}

fn set_input_number(label: char, value: u64, map: &mut HashMap<&str, bool>) {
    let num_bits = map.keys().filter(|k| k.starts_with(label)).count();
    let bits = format!("{value:b}").chars().collect_vec();

    // claude.ai came up with this. The borrow checker was driving me crazy with the stupid &str lifetimes.
    // apparently you can't update a hashmap in a loop, when you calculate the key of type &str inside that loop.
    // First collect all the keys we need to update
    let updates: Vec<_> = (0..num_bits)
        .map(|idx| format!("{label}{idx:02}"))
        .collect();

    // Then update them
    for (idx, key) in updates.iter().enumerate() {
        let bit_char = if idx >= bits.len() {
            &'0'
        } else {
            bits.get(bits.len() - idx - 1).unwrap_or(&'0')
        };
        let bit_value = *bit_char == '1';
        if let Some(bit) = map.get_mut(key.as_str()) {
            *bit = bit_value;
        }
    }
}

fn check_basic_functionality(
    gates: &[Gate],
    initial_map: &HashMap<&str, bool>,
    x_real_decimal: u64,
    y_real_decimal: u64,
) {
    let num_x_bits = initial_map.keys().filter(|k| k.starts_with("x")).count();
    let num_y_bits = initial_map.keys().filter(|k| k.starts_with("y")).count();
    let num_z_bits = gates.iter().filter(|g| g.out.starts_with("z")).count();

    assert_eq!(num_x_bits, num_y_bits);

    // set x to 1;
    let mut map: HashMap<&str, bool> = initial_map.clone();
    let y = 1;
    set_input_number('y', y, &mut map);

    let result = check_for_brokenness(
        "real_input".to_string(),
        x_real_decimal,
        y_real_decimal,
        &mut map,
        gates,
    );

    let broken = (0..=num_x_bits)
        .map(|exp| {
            let x = 2u64.pow(exp as u32) - 1;

            let res_1 = check_for_brokenness(format!("2^{exp} -1"), x, y, &mut map, gates);
            res_1

            // println!("2^{exp}; x: {x}; y: {y}; is_broken: {is_broken}; z_decimal_expected: {z_decimal_expected}; z_decimal_actual: {z_decimal_actual}; z_binary_actual: {z_binary_actual}; z_binary_expected: {z_binary_expected}");
            //
            // if is_broken {
            //     let broken_ranges =
            //         find_broken_ranges_in_bit_string(&z_binary_actual, &z_binary_expected);
            //     find_swaps(&gates, &map, &z_binary_expected, &broken_ranges);
            // }
        })
        .chain(vec![result])
        .filter_map(identity)
        .collect_vec();

    println!("found {} broken testcases", broken.len());
    dbg!(broken);
}

#[derive(Debug)]
struct BrokenTestResult {
    label: String,
    x: u64,
    y: u64,
    z_decimal_expected: u64,
    z_decimal_actual: u64,
    z_binary_expected: String,
    z_binary_actual: String,
}

fn check_for_brokenness<'a>(
    label: String,
    x: u64,
    y: u64,
    map: &mut HashMap<&'a str, bool>,
    gates: &[Gate<'a>],
) -> Option<BrokenTestResult> {
    set_input_number('x', x, map);
    set_input_number('y', y, map);
    //clear signals
    for g in gates.iter() {
        map.remove(&g.out);
    }

    let z_decimal_expected = x + y;
    let final_signals = evaluate_dag(&map, &gates);

    let (z_binary_actual, z_decimal_actual) =
        read_binary_number_from_register_starting_with_char("z", &final_signals);

    let z_binary_expected = format!(
        "{:0>width$b}",
        z_decimal_expected,
        width = z_binary_actual.len()
    );

    let is_broken = z_decimal_actual != z_decimal_expected;

    is_broken.then_some(BrokenTestResult {
        label,
        x,
        y,
        z_decimal_expected,
        z_decimal_actual,
        z_binary_expected,
        z_binary_actual,
    })
}

fn find_swaps(
    gates: &[Gate],
    initial_map: &HashMap<&str, bool>,
    expected_binary: &str,
    broken_ranges: &Vec<RangeInclusive<usize>>,
) {
    let mut gates: Vec<Gate> = gates.into_iter().cloned().collect_vec();
    let mut map = initial_map.clone();

    let mut improvements = Vec::with_capacity(100);

    (0..gates.len())
        .tuple_combinations()
        .for_each(|(idx_1, idx_2)| {
            //println!("Swapping {idx_1} and {idx_2}");
            swap(&mut gates, idx_1, idx_2);
            //clear signals
            for g in gates.iter() {
                map.remove(&g.out);
            }
            let new_map = evaluate_dag(&map, &gates);
            let (z_binary, z_decimal) =
                read_binary_number_from_register_starting_with_char("z", &new_map);

            swap(&mut gates, idx_2, idx_1);

            if z_binary.len() == expected_binary.len() {
                let diffs = find_broken_ranges_in_bit_string(&z_binary, expected_binary);
                if diffs.len() < broken_ranges.len() {
                    if diffs
                        .iter()
                        .all(|diff_range| broken_ranges.contains(diff_range))
                    {
                        improvements.push((idx_1, idx_2, z_binary, z_decimal, diffs));
                    }
                }
            }
        });

    println!(
        "found {} swap-combinations that improve the result",
        improvements.len()
    );
    for (idx_1, idx_2, z_binary, z_decimal, diff_ranges) in
        improvements
            .iter()
            .sorted_by_key(|(idx_1, idx_2, z_binary, z_decimal, diff_ranges)| {
                (diff_ranges.len(), idx_1, idx_2)
            })
    {
        println!(
            "num_diffs: {}; diffs: {diff_ranges:?}; idx_1: {idx_1}; idx_2: {idx_2}; z_binary: {z_binary}; z_decimal: {z_decimal}",
            diff_ranges.len()
        );
    }
}

fn swap(gates: &mut Vec<Gate>, idx_1: usize, idx_2: usize) {
    let tmp_1 = gates[idx_1].out.clone();
    let tmp_2 = gates[idx_2].out.clone();
    if let Some(gate_1) = gates.get_mut(idx_1) {
        gate_1.out = tmp_2;
    }
    if let Some(gate_2) = gates.get_mut(idx_2) {
        gate_2.out = tmp_1;
    }
}

fn render_graph(gates: &[Gate], current_map: &HashMap<&str, bool>) {
    let node_names: HashMap<&str, String> = gates
        .iter()
        .map(|gate| {
            (
                gate.out.clone(),
                format!(
                    "{}\n{}",
                    gate.out,
                    match gate.op {
                        Operator::AND => "AND",
                        Operator::OR => "OR",
                        Operator::XOR => "XOR",
                    }
                ),
            )
        })
        .collect::<HashMap<&str, String>>();

    // Create edges with owned Strings instead of references
    let edges = gates
        .iter()
        .flat_map(
            |Gate {
                 in_1,
                 in_2,
                 op,
                 out,
             }| {
                [in_1, in_2]
                    .into_iter()
                    .map(|input| {
                        (
                            node_names.get(input).map(|v| v.as_str()).unwrap_or(input),
                            node_names.get(out).map(|v| v.as_str()).unwrap_or(out),
                            current_map.get(input).unwrap(),
                        )
                    })
                    .collect::<Vec<_>>()
            },
        )
        .collect_vec();

    // Create the graph with owned Strings
    let g = DiGraphMap::<&str, bool>::from_edges(edges);

    println!("{}", Dot::with_config(&g, &[]));
}
fn find_broken_ranges_in_bit_string(actual: &str, expected: &str) -> Vec<RangeInclusive<usize>> {
    /*
                  idx: 543210987|65|4321098765|43210987|654|321|0|9876543210
             z_binary: 100001100|10|0000000000|01010111|011|100|0|0001110000
    z_binary_expected: 100001100|01|0111111111|01010111|100|100|1|0001110000
       */
    actual
        .chars()
        .rev()
        .zip(expected.chars().rev())
        .enumerate()
        .map(|(idx, (a, e))| (idx, a, e, a == e))
        .chunk_by(|(_, _, _, is_equal)| *is_equal)
        .into_iter()
        .map(|(c, group)| {
            let entries = group.collect_vec();
            let min = entries.iter().map(|t| t.0).min().unwrap();
            let max = entries.iter().map(|t| t.0).max().unwrap();
            (c, min..=max)
        })
        .inspect(|t| debug!("{t:?}"))
        .filter_map(|t| (t.0 == false).then_some(t.1))
        .collect_vec()
}

fn read_binary_number_from_register_starting_with_char(
    starts_with_pattern: &str,
    signals: &HashMap<&str, bool>,
) -> (String, u64) {
    let binary: String = signals
        .into_iter()
        .filter(|(key, _)| key.starts_with(starts_with_pattern))
        .sorted_by_key(|(key, _)| key.clone())
        .rev() // most significant bit first
        .map(|(_, value)| (*value as u8).to_string())
        .collect();

    let result = u64::from_str_radix(&binary, 2).unwrap();
    (binary, result)
}

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

#[derive(Debug, Clone)]
struct Gate<'a> {
    in_1: &'a str,
    in_2: &'a str,
    op: Operator,
    out: &'a str,
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

fn parse(input: &str) -> IResult<&str, (HashMap<&str, bool>, Vec<Gate>)> {
    let (input, initial_map) = separated_list1(
        line_ending,
        separated_pair(
            alphanumeric1,
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
                    alphanumeric1,
                    alt((
                        value(Operator::AND, tag(" AND ")),
                        value(Operator::OR, tag(" OR ")),
                        value(Operator::XOR, tag(" XOR ")),
                    )),
                    alphanumeric1,
                )),
                tag(" -> "),
                alphanumeric1,
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
    initial_signals: &'a HashMap<&'a str, bool>,
    gates: &'a [Gate<'a>],
) -> HashMap<&'a str, bool> {
    let mut signals = initial_signals.clone();

    let mut open_list_gates: VecDeque<Gate> = VecDeque::from_iter(gates.into_iter().cloned());

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
                signal_in_1, op, signal_in_2, gate.out, in_1, op, in_2, out
            )
        } else {
            debug!("No open pos found");
            return signals;
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
    fn test_set_and_read_number() -> miette::Result<()> {
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

ntg XOR fgs -> mjb"#
            .trim();

        let (_, (mut signals, _)) = parse(input).map_err(|e| miette!("parse failed {}", e))?;

        for i in 0..32 {
            set_input_number('x', i, &mut signals);
            let (_, result) = read_binary_number_from_register_starting_with_char("x", &signals);
            assert_eq!(result, i);
        }

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
