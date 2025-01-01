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
use rand::Rng;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::{Display, Formatter};
use std::ops::{BitAnd, BitOr, BitXor, Not, RangeInclusive};
use tracing::{debug, info};

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, (initial_signals, gates)): (&str, (HashMap<&str, bool>, Vec<Gate>)) =
        parse(input).map_err(|e| miette!("parse failed {}", e))?;

    // Task is to fix the gates to perform the addition operation correctly.
    // x + y = z

    let gates = gates;

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
    // debug!("wires_affected_by_swaps: {wires_affected_by_swaps}");

    let final_signals = evaluate_dag(&initial_signals, &gates);

    let (x_binary, x_decimal) =
        read_binary_number_from_register_starting_with_char("x", &final_signals);
    let (y_binary, y_decimal) =
        read_binary_number_from_register_starting_with_char("y", &final_signals);
    let (z_binary, z_decimal_actual) =
        read_binary_number_from_register_starting_with_char("z", &final_signals);

    let z_decimal_expected = x_decimal + y_decimal;
    let z_binary_expected = format!("{:b}", z_decimal_expected);

    debug!("");
    debug!("");
    debug!("         x_decimal: {x_decimal}");
    debug!("         y_decimal: {y_decimal}");
    debug!("z_decimal_expected: {z_decimal_expected}");
    debug!("  z_decimal_actual: {z_decimal_actual}");

    debug!("          x_binary: {x_binary}");
    debug!("          y_binary: {y_binary}");
    debug!("          z_binary: {z_binary}");
    debug!(" z_binary_expected: {z_binary_expected}");

    let is_match = z_decimal_expected == z_decimal_actual;
    debug!("did I fix it? {is_match}");

    // let broken_ranges: Vec<RangeInclusive<usize>> =
    //     find_broken_ranges_in_bit_string(&z_binary, &z_binary_expected);
    //
    // debug!("broken_ranges: {:?}", broken_ranges);

    let mut signals = initial_signals.clone();
    //render_graph(&gates, &final_signals);

    let num_x_bits = signals.keys().filter(|k| k.starts_with("x")).count();
    let num_y_bits = signals.keys().filter(|k| k.starts_with("y")).count();

    let testcases = (0..num_x_bits)
        .flat_map(|exp| {
            let x = 0;
            let y = 2u64.pow(exp as u32);

            vec![(format!("0 + 2^{exp}"), x, y)]
        })
        .chain(vec![("real_input".to_string(), x_decimal, y_decimal)])
        .collect_vec();

    let result = fix_gates(&gates, &mut signals, testcases).unwrap();
    Ok(result)
}

fn determine_gate_connections<'a>(
    broken_bit_range: RangeInclusive<usize>,
    gates: &'a [Gate<'a>],
    signals: &mut HashMap<&'a str, bool>,
) -> HashSet<Gate<'a>> {
    let lower_bit_idx = broken_bit_range.clone().min().unwrap();
    let higher_bit_idx = broken_bit_range.max().unwrap();
    assert_eq!(higher_bit_idx - lower_bit_idx, 1);

    let x_label = format!("x{lower_bit_idx:02}");
    let z_label_lower = format!("z{lower_bit_idx:02}");
    let z_label_higher = format!("z{higher_bit_idx:02}");

    let gates_connected_from_lower_input_x =
        find_gates_connected_down_from_signal(x_label.clone(), gates);

    let gates_connected_to_lower_output =
        find_gates_connected_up_from_output(z_label_lower.clone(), gates);
    let gates_connected_to_higher_output =
        find_gates_connected_up_from_output(z_label_higher.clone(), gates);

    let gates_propagating_to_outputs = gates_connected_to_lower_output
        .union(&gates_connected_to_higher_output)
        .cloned()
        .collect::<HashSet<_>>();

    let relevant_gates = [
        gates_connected_from_lower_input_x.clone(),
        gates_propagating_to_outputs.clone(),
    ]
    .iter()
    .cloned()
    .reduce(|a, b| a.intersection(&b).cloned().collect())
    .unwrap_or_default();

    debug!(
        "{x_label} propagates to {} gates",
        gates_connected_from_lower_input_x.len()
    );
    debug!(
        "{z_label_lower} {z_label_higher} propagate from {} gates",
        gates_propagating_to_outputs.len()
    );
    debug!("Overlap: {} gates", &relevant_gates.len());
    //dbg!(relevant_gates);

    relevant_gates
}

fn find_gates_connected_down_from_signal<'a>(
    label: String,
    gates: &'a [Gate<'a>],
) -> HashSet<Gate<'a>> {
    debug!("find_gates_connected_down_from_signal(label: {label})");

    let mut open_list = VecDeque::from([label]);
    let mut affected_gates: HashSet<Gate> = HashSet::new();
    while let Some(current) = open_list.pop_front() {
        let relevant_gates: HashSet<Gate> = gates
            .iter()
            .filter(|g| g.in_1 == current || g.in_2 == current)
            .cloned()
            .collect();
        for g in relevant_gates.iter() {
            affected_gates.insert(*g);
            open_list.push_back(g.out.to_string())
        }
    }

    affected_gates
}

fn find_gates_connected_up_from_output<'a>(
    label: String,
    gates: &'a [Gate<'a>],
) -> HashSet<Gate<'a>> {
    debug!("find_gates_connected_up_from_output(label: {label})");

    let mut open_list = VecDeque::from([label]);
    let mut affected_gates: HashSet<Gate> = HashSet::new();
    while let Some(current) = open_list.pop_front() {
        let relevant_gates: HashSet<Gate> =
            gates.iter().filter(|g| g.out == current).cloned().collect();
        for g in relevant_gates.iter() {
            affected_gates.insert(*g);
            open_list.push_back(g.in_1.to_string());
            open_list.push_back(g.in_2.to_string());
        }
    }

    affected_gates
}

fn fix_gates<'a>(
    gates: &'a [Gate<'a>],
    initial_map: &mut HashMap<&'a str, bool>,
    testcases: Vec<(String, u64, u64)>,
) -> Option<String> {
    let (_, total_num_wrong_bits) =
        run_testcases_and_count_total_number_of_wrong_bits(gates, initial_map, &testcases);

    let correct_swap_indices = find_swaps(gates, initial_map, total_num_wrong_bits, &testcases)?;
    let affected_indices: HashSet<usize> = correct_swap_indices
        .into_iter()
        .flat_map(|(from, to)| vec![from, to])
        .collect();

    let wires_csv = gates
        .iter()
        .enumerate()
        .filter_map(|(idx, gate)| affected_indices.contains(&idx).then_some(gate.out))
        .sorted()
        .join(",");

    Some(wires_csv)
}

fn find_swaps<'a>(
    original_gates: &'a [Gate<'a>],
    signals: &mut HashMap<&'a str, bool>,
    total_num_wrong_bits: usize,
    testcases: &[(String, u64, u64)],
) -> Option<Vec<(usize, usize)>> {
    let mut gates: Vec<Gate<'a>> = original_gates.iter().cloned().collect_vec();

    let (baseline_broken_results, baseline_current_total_num_wrong_bits) =
        run_testcases_and_count_total_number_of_wrong_bits(&gates, signals, testcases);

    let num_x_bits = signals.keys().filter(|k| k.starts_with("x")).count();
    let random_testcases = generate_random_testcases(100, num_x_bits);

    //debug!("{random_testcases:?}");

    debug!("Trying to find swaps. baseline_current_total_num_wrong_bits: {baseline_current_total_num_wrong_bits}");
    for b @ BrokenTestResult {
        label,
        x,
        y,
        z_decimal_expected,
        z_decimal_actual,
        z_binary_expected,
        z_binary_actual,
        broken_ranges,
    } in baseline_broken_results.iter()
    {
        let total_bits_wrong = b.total_bits_wrong();
        debug!("label: {label}; num_bits_wrong: {total_bits_wrong}; broken_ranges: {broken_ranges:?}; x: {x}; y: {y}; z_decimal_expected: {z_decimal_expected}; z_decimal_actual: {z_decimal_actual}; z_binary_expected: {z_binary_expected}; z_binary_actual: {z_binary_actual}");
    }

    let swap_candidates_per_bit: Vec<Vec<(usize, Gate)>> = baseline_broken_results
        .iter()
        .filter(|b| b.broken_ranges.len() == 1)
        .map(|broken_case| {
            let range = broken_case.broken_ranges.first().unwrap();
            determine_gate_connections(range.clone(), original_gates, signals)
                .into_iter()
                .map(|g| {
                    let idx = original_gates.iter().position(|g2| &g == g2).unwrap();
                    (idx, g)
                })
                .collect_vec()
        })
        .collect_vec();

    debug!(
        "Found swap candidates for {} bits.",
        swap_candidates_per_bit.len()
    );
    for swap_candidates in swap_candidates_per_bit.iter() {
        let swap_candidates_str = swap_candidates
            .iter()
            .map(|(idx, g)| format!("{idx}({})", g.out))
            .join(", ");
        debug!(
            "{} swap_candidates: {}",
            swap_candidates.len(),
            swap_candidates_str
        );
    }

    let swaps_groups = swap_candidates_per_bit
        .iter()
        .map(|swaps| {
            swaps
                .iter()
                .map(|(idx, _)| idx)
                .tuple_combinations()
                .collect::<Vec<(_, _)>>()
        })
        .collect_vec();

    for (idx, swap_group) in swaps_groups.iter().enumerate() {
        debug!(
            "#{idx} swap_group.len() = {} - {:?}",
            swap_group.len(),
            swap_group
        )
    }

    let total_swap_combinations = swaps_groups.iter().multi_cartesian_product().count();

    debug!("Found {total_swap_combinations} total swap combinations");
    debug!("Checking against baseline of {baseline_current_total_num_wrong_bits} broken bits");

    let improving_swap_combinations_per_bit = swaps_groups
        .iter()
        .map(|swaps| {
            swaps
                .iter()
                .filter(|(swap_from, swap_to)| {
                    swap(&mut gates, **swap_from, **swap_to);
                    let (new_result, new_total_number_of_broken_bits) =
                        run_testcases_and_count_total_number_of_wrong_bits(
                            &gates, signals, testcases,
                        );
                    swap(&mut gates, **swap_to, **swap_from);

                    new_total_number_of_broken_bits < baseline_current_total_num_wrong_bits
                })
                .collect_vec()
        })
        .collect_vec();

    for (idx, swap_group) in improving_swap_combinations_per_bit.iter().enumerate() {
        debug!(
            "#{idx} improving_swap_group.len() = {} - {:?}",
            swap_group.len(),
            swap_group
        )
    }

    let total_improving_swap_combinations = improving_swap_combinations_per_bit
        .iter()
        .multi_cartesian_product()
        .count();

    debug!("Found {total_improving_swap_combinations} improving_swap_combinations");

    for (idx, swaps) in improving_swap_combinations_per_bit
        .iter()
        .multi_cartesian_product()
        .enumerate()
    {
        //swap
        for (swap_from, swap_to) in &swaps {
            swap(&mut gates, **swap_from, **swap_to);
        }
        let (_, new_total_number_of_broken_bits) =
            run_testcases_and_count_total_number_of_wrong_bits(&gates, signals, testcases);

        if new_total_number_of_broken_bits == 0 {
            debug!(
                "found solution after {} checks: {swaps:?}; Checking random {} testcases",
                idx + 1,
                random_testcases.len()
            );
            let (_, new_total_number_of_broken_bits) =
                run_testcases_and_count_total_number_of_wrong_bits(
                    &gates,
                    signals,
                    &random_testcases,
                );
            if new_total_number_of_broken_bits == 0 {
                debug!("found solution after checking random values as well");
                return Some(
                    swaps
                        .into_iter()
                        .map(|(from, to)| (**from, **to))
                        .collect_vec(),
                );
            } else {
                debug!("discarding solution after checking random values. Found {new_total_number_of_broken_bits} broken bits");
            }
        }

        //swap back
        for (swap_from, swap_to) in &swaps {
            swap(&mut gates, **swap_to, **swap_from);
        }
    }
    None
}

fn generate_random_testcases(n: u32, num_x_bits: usize) -> Vec<(String, u64, u64)> {
    (0..n)
        .flat_map(|_| {
            let mut x: u64 = 0;
            let mut y: u64 = 0;

            (0..num_x_bits).for_each(|_| {
                x = (x << 1) + rand::rng().random_range(0..=1);
                y = (y << 1) + rand::rng().random_range(0..=1);
            });

            vec![(format!("{x} + {y}"), x, y)]
        })
        .collect_vec()
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

fn run_testcases_and_count_total_number_of_wrong_bits<'a>(
    gates: &[Gate<'a>],
    map: &mut HashMap<&'a str, bool>,
    testcases: &[(String, u64, u64)],
) -> (Vec<BrokenTestResult>, usize) {
    let num_x_bits = map.keys().filter(|k| k.starts_with("x")).count();
    let num_y_bits = map.keys().filter(|k| k.starts_with("y")).count();
    let num_z_bits = gates.iter().filter(|g| g.out.starts_with("z")).count();

    assert_eq!(num_x_bits, num_y_bits);

    let broken = testcases
        .iter()
        .filter_map(|(label, x, y)| check_for_brokenness(label.to_string(), *x, *y, map, gates))
        .collect_vec();

    info!("found {} broken testcases", broken.len());
    for b @ BrokenTestResult {
        label,
        x,
        y,
        z_decimal_expected,
        z_decimal_actual,
        z_binary_expected,
        z_binary_actual,
        ..
    } in broken.iter()
    {
        let total_bits_wrong = b.total_bits_wrong();
        info!("label: {label}; num_bits_wrong: {total_bits_wrong}; x: {x}; y: {y}; z_decimal_expected: {z_decimal_expected}; z_decimal_actual: {z_decimal_actual}; z_binary_expected: {z_binary_expected}; z_binary_actual: {z_binary_actual}");
    }

    let total_num_broken_bits = broken.iter().map(|b| b.total_bits_wrong()).sum();

    (broken, total_num_broken_bits)
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
    broken_ranges: Vec<RangeInclusive<usize>>,
}

impl BrokenTestResult {
    pub(crate) fn total_bits_wrong(&self) -> usize {
        self.broken_ranges
            .iter()
            .map(|r| r.try_len().unwrap())
            .sum()
    }
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
    let final_signals = evaluate_dag(map, gates);

    let (z_binary_actual, z_decimal_actual) =
        read_binary_number_from_register_starting_with_char("z", &final_signals);

    let z_binary_expected = format!(
        "{:0>width$b}",
        z_decimal_expected,
        width = z_binary_actual.len()
    );

    let is_broken = z_decimal_actual != z_decimal_expected;

    is_broken.then(|| {
        let broken_ranges = find_broken_ranges_in_bit_string(&z_binary_actual, &z_binary_expected);
        BrokenTestResult {
            label,
            x,
            y,
            z_decimal_expected,
            z_decimal_actual,
            z_binary_expected,
            z_binary_actual,
            broken_ranges,
        }
    })
}

fn swap(gates: &mut Vec<Gate>, idx_1: usize, idx_2: usize) {
    let tmp_1 = gates[idx_1].out;
    let tmp_2 = gates[idx_2].out;
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
                gate.out,
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

    debug!("{}", Dot::with_config(&g, &[]));
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
        .filter_map(|t| (!t.0).then_some(t.1))
        .collect_vec()
}

fn read_binary_number_from_register_starting_with_char(
    starts_with_pattern: &str,
    signals: &HashMap<&str, bool>,
) -> (String, u64) {
    let binary: String = signals
        .iter()
        .filter(|(key, _)| key.starts_with(starts_with_pattern))
        .sorted_by_key(|(key, _)| key.clone())
        .rev() // most significant bit first
        .map(|(_, value)| (*value as u8).to_string())
        .collect();

    let result = u64::from_str_radix(&binary, 2).unwrap();
    (binary, result)
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
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

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]

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

    let mut open_list_gates: VecDeque<Gate> = VecDeque::from_iter(gates.iter().cloned());

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
            signals.insert(gate.out, out);
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
