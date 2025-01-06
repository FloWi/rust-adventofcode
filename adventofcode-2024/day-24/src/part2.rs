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
use rand::Rng;
use std::collections::{HashMap, HashSet, VecDeque};
use std::ops::{BitAnd, BitOr, BitXor};
use tracing::{debug, info};

pub fn process(input: &str) -> miette::Result<String> {
    let (_, original_aoc_computer) = parse(input).map_err(|e| miette!("parse failed {}", e))?;

    let mut aoc_computer = original_aoc_computer.clone();
    let x = original_aoc_computer.x;
    let y = original_aoc_computer.y;

    aoc_computer.run_computer();
    let broken_bits = find_broken_output_bits(&mut aoc_computer);
    assert_eq!(broken_bits.len(), 4);
    info!(
        "Found {} broken bits at indices {:?}",
        broken_bits.len(),
        broken_bits
    );

    let swap_candidates_per_bit: Vec<(usize, HashSet<IndexedGate>)> =
        aoc_computer.determine_intersecting_gates_for_broken_bits(broken_bits);

    let random_testcases = generate_random_testcases(100, aoc_computer.num_z_bits - 1);
    let real_testcase = vec![(format!("{} + {} = {}", x, y, x + y), x, y)];
    let one_bit_testcases = generate_one_bit_testcases(&mut aoc_computer);
    let all_testcases = random_testcases
        .into_iter()
        .chain(real_testcase)
        .collect_vec();

    let reduced_swap_groups = narrow_down_swap_groups(
        &mut aoc_computer,
        swap_candidates_per_bit,
        &one_bit_testcases,
    );

    let final_candidates = reduced_swap_groups
        .iter()
        .map(|(_bit, swap_groups)| swap_groups)
        .multi_cartesian_product()
        .collect_vec();

    info!(
        "Performing final check on {} candidates",
        final_candidates.len()
    );

    let result = final_candidates
        .iter()
        .enumerate()
        .find_map(|(idx, group)| {
            // swap
            for (from, to) in group {
                aoc_computer.swap_gate_outputs(*from, *to);
            }
            let num_broken = run_testcases_and_count_broken_bits(&mut aoc_computer, &all_testcases);

            //swap back
            for (from, to) in group {
                aoc_computer.swap_gate_outputs(*from, *to);
            }
            if num_broken == 0 {
                info!(
                    "FOUND solution after {} of {} checks",
                    idx + 1,
                    final_candidates.len()
                );
            };
            (num_broken == 0).then(|| {
                group
                    .iter()
                    .flat_map(|(idx_1, idx_2)| {
                        let gate_1 = aoc_computer.indexed_gates.get(*idx_1).unwrap();
                        let gate_2 = aoc_computer.indexed_gates.get(*idx_2).unwrap();
                        let label_1 = aoc_computer.get_name_name_from_memory_location(&gate_1.out);
                        let label_2 = aoc_computer.get_name_name_from_memory_location(&gate_2.out);
                        vec![label_1, label_2]
                    })
                    .sorted()
                    .join(",")
            })
        })
        .expect("There should be a solution");

    Ok(result)
}

fn narrow_down_swap_groups(
    aoc_computer: &mut AocComputer,
    original_swap_candidates_per_bit: Vec<(usize, HashSet<IndexedGate>)>,
    testcases: &[(String, u64, u64)],
) -> Vec<(usize, Vec<(usize, usize)>)> {
    debug!(
        "original_swap_candidates_per_bit: {:?}",
        &original_swap_candidates_per_bit
    );
    // find the swaps per group, that improve from the baseline
    let baseline_broken_bits = run_testcases_and_count_broken_bits(aoc_computer, testcases);
    let swap_groups_with_index_positions: Vec<(usize, Vec<usize>)> =
        original_swap_candidates_per_bit
            .iter()
            .map(|(bit_idx, indexed_gates)| {
                let gates_with_indexes: Vec<usize> = indexed_gates
                    .iter()
                    .map(|ig| {
                        aoc_computer
                            .indexed_gates
                            .iter()
                            .position(|curr| curr == ig)
                            .unwrap()
                    })
                    .collect_vec();
                (*bit_idx, gates_with_indexes)
            })
            .collect_vec();

    debug!(
        "swap_groups_with_index_positions: {:?}",
        &swap_groups_with_index_positions
    );
    let initial_swap_index_candidates = swap_groups_with_index_positions
        .iter()
        .flat_map(|(_, indices)| indices)
        .unique()
        .sorted()
        .collect_vec();
    debug!(
        "{} initial_swap_index_candidates: {:?}",
        &initial_swap_index_candidates.len(),
        &initial_swap_index_candidates
    );

    let relevant_swap_groups_with_index_positions = swap_groups_with_index_positions
        .into_iter()
        .map(|(bit_idx, indexes_of_swap_candidates)| {
            debug!("Checking swaps for bit #{bit_idx}");
            let relevant_swap_candidates = indexes_of_swap_candidates
                .into_iter()
                .tuple_combinations()
                .filter_map(|(from, to)| {
                    debug!("Checking swap {from}-{to} for bit #{bit_idx}");
                    // debug!("Gates after swap: \n===========================================================================================================");
                    // aoc_computer.create_gates_from_indexed_gates().iter().for_each(|g| debug!("{} {:?} {} -> {}", g.in_1, g.op, g.in_2, g.out) );

                    aoc_computer.swap_gate_outputs(from, to);
                    let num_broken = run_testcases_and_count_broken_bits(aoc_computer, testcases);
                    aoc_computer.swap_gate_outputs(from, to);
                    let has_improved = num_broken < baseline_broken_bits;
                    if has_improved {
                        debug!("Found improvement by swapping gates at indices {from} and {to}. Broken: {num_broken} vs. baseline {baseline_broken_bits}");
                    } else {
                        debug!("No improvement by swapping gates at indices {from} and {to}. Broken: {num_broken} vs. baseline {baseline_broken_bits}");
                    }
                    has_improved.then_some((from, to))
                })
                .collect_vec();
            (bit_idx, relevant_swap_candidates)
        })
        .collect_vec();

    debug!(
        "relevant_swap_groups_with_index_positions: {:?}",
        &relevant_swap_groups_with_index_positions
    );

    let relevant_swap_index_candidates = relevant_swap_groups_with_index_positions
        .iter()
        .flat_map(|(_, swap_pairs)| swap_pairs.iter().flat_map(|(from, to)| vec![from, to]))
        .unique()
        .sorted()
        .collect_vec();
    debug!(
        "{} relevant_swap_index_candidates: {:?}",
        relevant_swap_index_candidates.len(),
        &relevant_swap_index_candidates
    );

    relevant_swap_groups_with_index_positions
}

fn generate_random_testcases(n: usize, num_x_bits: usize) -> Vec<(String, u64, u64)> {
    (0..n)
        .flat_map(|_| {
            let mut x: u64 = 0;
            let mut y: u64 = 0;

            (0..num_x_bits).for_each(|_| {
                x = (x << 1) + rand::thread_rng().gen_range(0..=1);
                y = (y << 1) + rand::thread_rng().gen_range(0..=1);
            });

            vec![(format!("{x} + {y}"), x, y)]
        })
        .collect_vec()
}

fn run_testcases_and_count_broken_bits(
    aoc_computer: &mut AocComputer,
    testcases: &[(String, u64, u64)],
) -> u32 {
    testcases
        .iter()
        .map(|(label, test_x, test_y)| {
            let expected_z = test_x + test_y;
            aoc_computer.reset();
            aoc_computer.x = *test_x;
            aoc_computer.y = *test_y;
            debug!("Running computer with x={} and y={}", test_x, test_y);
            aoc_computer.run_computer();
            debug!(
                "Done running computer with x={} and y={}. Result z={}",
                test_x, test_y, aoc_computer.z
            );
            let actual_z = aoc_computer.z;

            let diff = actual_z ^ expected_z;
            let num_broken = diff.count_ones();
            debug!("Testcase {label}: Num broken: {num_broken}");
            num_broken
        })
        .sum()
}

fn generate_one_bit_testcases(aoc_computer: &mut AocComputer) -> Vec<(String, u64, u64)> {
    (0..(aoc_computer.num_z_bits - 1))
        .map(|idx| {
            let test_x = 0;
            let test_y = 1 << idx;

            (
                format!("0 + 2^({}) --> 0 + {test_y}", idx + 1),
                test_x,
                test_y,
            )
        })
        .collect_vec()
}

fn find_broken_output_bits(aoc_computer: &mut AocComputer) -> Vec<usize> {
    (0..(aoc_computer.num_z_bits - 1))
        .filter(|idx| {
            let test_x = 0;
            let test_y = 1 << idx;
            let expected_z = test_x + test_y;
            aoc_computer.reset();
            aoc_computer.x = test_x;
            aoc_computer.y = test_y;
            aoc_computer.run_computer();
            let actual_z = aoc_computer.z;

            aoc_computer.debug_print(Some(expected_z));

            actual_z != expected_z
        })
        .collect_vec()
}

#[derive(Clone)]
struct AocComputer {
    gates: Vec<Gate>,
    initial_signals: HashMap<String, bool>,
    x: u64,
    y: u64,
    z: u64,
    intermediate_signals: Vec<Option<u8>>,
    intermediate_signal_names: Vec<String>,
    intermediate_mapping: HashMap<String, usize>,
    indexed_gates: Vec<IndexedGate>,
    num_z_bits: usize,
}

impl AocComputer {
    fn swap_gate_outputs(&mut self, index_gate_pos_1: usize, index_gate_pos_2: usize) {
        // claude came up with this.
        // Unfortunately acquiring two mutable instances at the same time doesn't work
        // get_many_mut exists, but is an unstable feature, so I stick with index arithmetics. YOLO

        let (min_idx, max_idx) = if index_gate_pos_1 < index_gate_pos_2 {
            (index_gate_pos_1, index_gate_pos_2)
        } else {
            (index_gate_pos_2, index_gate_pos_1)
        };

        // Split the slice at the smaller index
        let (left, right) = self.indexed_gates.split_at_mut(min_idx + 1);
        // Now we can safely get mutable references to both elements
        let first = &mut left[min_idx];
        let second = &mut right[max_idx - (min_idx + 1)];

        std::mem::swap(&mut first.out, &mut second.out);
    }

    fn debug_print(&self, expected_z: Option<u64>) {
        let num_z_bits = self.num_z_bits;
        let x = self.x;
        let y = self.y;

        let actual_z = self.z;
        debug!("         x: {x}");
        debug!("         y: {y}");
        debug!("  actual_z: {actual_z}");
        if let Some(expected_z) = expected_z {
            debug!("expected_z: {expected_z}");
        }

        debug!("         x: {:0>width$b}", x, width = num_z_bits);
        debug!("         y: {:0>width$b}", y, width = num_z_bits);
        debug!("  actual_z: {:0>width$b}", actual_z, width = num_z_bits);
        if let Some(expected_z) = expected_z {
            debug!("expected_z: {:0>width$b}", expected_z, width = num_z_bits);
        }
    }

    /// Checks the connections between x_n, z_n and z_n+1
    /// x_n INTERSECT (z_n UNION z_n+1)
    /// should be 6 gates for every bit
    fn determine_intersecting_gates_for_broken_bits(
        &mut self,
        broken_z_bits: Vec<usize>,
    ) -> Vec<(usize, HashSet<IndexedGate>)> {
        broken_z_bits
            .into_iter()
            .map(|idx| {
                let x_mem_location = MemoryLocation::X(idx);
                let z_lo_mem_location = MemoryLocation::Z(idx);
                let z_hi_mem_location = MemoryLocation::Z(idx + 1);

                // all gates that are connected to x
                let gates_connected_from_x =
                    self.find_gates_connected_down_from_signal(&x_mem_location);

                // all gates that are connected to z_lo and z_hi
                let gates_propagating_to_lo_output =
                    self.find_gates_connected_up_from_output(&z_lo_mem_location);
                let gates_propagating_to_hi_output =
                    self.find_gates_connected_up_from_output(&z_hi_mem_location);

                let gates_connected_to_both_z = gates_propagating_to_lo_output
                    .union(&gates_propagating_to_hi_output)
                    .cloned()
                    .collect::<HashSet<_>>();

                let relevant_gates = [
                    gates_connected_from_x.clone(),
                    gates_connected_to_both_z.clone(),
                ]
                .iter()
                .cloned()
                .reduce(|a, b| a.intersection(&b).cloned().collect())
                .unwrap_or_default();

                info!(
                    "{x_mem_location:?} propagates to {} gates",
                    gates_connected_from_x.len()
                );
                info!(
                    "{z_lo_mem_location:?} {z_hi_mem_location:?} propagate from {} gates",
                    gates_connected_to_both_z.len()
                );
                info!("Overlap: {} gates", &relevant_gates.len());

                assert_eq!(relevant_gates.len(), 6);
                (idx, relevant_gates)
            })
            .collect_vec()
    }

    fn find_gates_connected_down_from_signal(
        &self,
        memory_location: &MemoryLocation,
    ) -> HashSet<IndexedGate> {
        let mut open_list = VecDeque::from([memory_location.clone()]);
        let mut affected_gates: HashSet<IndexedGate> = HashSet::new();

        while let Some(current) = open_list.pop_front() {
            let relevant_gates: HashSet<IndexedGate> = self
                .indexed_gates
                .iter()
                .filter(|g| g.in_1 == current || g.in_2 == current)
                .cloned()
                .collect();
            for g in relevant_gates.iter() {
                affected_gates.insert(g.clone());
                open_list.push_back(g.out.clone())
            }
        }

        affected_gates
    }

    fn find_gates_connected_up_from_output(
        &self,
        memory_location: &MemoryLocation,
    ) -> HashSet<IndexedGate> {
        let mut open_list = VecDeque::from([memory_location.clone()]);
        let mut affected_gates: HashSet<IndexedGate> = HashSet::new();

        while let Some(current) = open_list.pop_front() {
            let relevant_gates: HashSet<IndexedGate> = self
                .indexed_gates
                .iter()
                .filter(|g| g.out == current)
                .cloned()
                .collect();
            for g in relevant_gates.iter() {
                affected_gates.insert(g.clone());
                open_list.push_back(g.in_1.clone());
                open_list.push_back(g.in_2.clone());
            }
        }

        affected_gates
    }

    fn new(gates: Vec<Gate>, input_signals: HashMap<String, bool>) -> Self {
        let intermediate_signal_names = gates
            .iter()
            .flat_map(|g| [g.in_1.clone(), g.in_2.clone(), g.out.clone()])
            .filter(|signal_name| {
                !signal_name.starts_with('x')
                    && !signal_name.starts_with('y')
                    && !signal_name.starts_with('z')
            })
            .unique()
            .sorted()
            .collect_vec();

        let mut intermediate_signals = Vec::with_capacity(intermediate_signal_names.len());
        for _ in 0..intermediate_signal_names.len() {
            intermediate_signals.push(None);
        }
        let intermediate_index_mapping: HashMap<String, usize> = intermediate_signal_names
            .iter()
            .enumerate()
            .map(|(idx, name)| (name.clone(), idx))
            .collect();

        let indexed_gates = gates
            .iter()
            .map(|g| {
                let in_1 = Self::get_register(&g.in_1, &intermediate_index_mapping);
                let in_2 = Self::get_register(&g.in_2, &intermediate_index_mapping);
                let out = Self::get_register(&g.out, &intermediate_index_mapping);
                IndexedGate {
                    in_1,
                    in_2,
                    op: g.op,
                    out,
                }
            })
            .collect_vec();

        let num_z_bits = gates
            .iter()
            .flat_map(|g| [g.in_1.clone(), g.in_2.clone(), g.out.clone()])
            .filter(|signal_name| signal_name.starts_with('z'))
            .unique()
            .count();

        let mut computer = Self {
            gates,
            initial_signals: input_signals.clone(),
            x: 0,
            y: 0,
            z: 0,
            intermediate_signals,
            intermediate_signal_names,
            intermediate_mapping: intermediate_index_mapping.clone(),
            indexed_gates,
            num_z_bits,
        };

        //initialize x and y using the mechanics we already have in place
        for (signal_name, value) in input_signals {
            let memory_location =
                AocComputer::get_register(&signal_name, &intermediate_index_mapping);
            computer.set(&memory_location, if value { 1 } else { 0 });
        }
        computer
    }

    fn reset(&mut self) {
        self.x = 0;
        self.y = 0;
        self.z = 0;

        for elem in self.intermediate_signals.iter_mut() {
            *elem = None;
        }
    }

    fn get_register(
        signal_name: &str,
        intermediate_index_mapping: &HashMap<String, usize>,
    ) -> MemoryLocation {
        match signal_name.split_at(1) {
            ("x", idx) => MemoryLocation::X(idx.parse::<usize>().unwrap()),
            ("y", idx) => MemoryLocation::Y(idx.parse::<usize>().unwrap()),
            ("z", idx) => MemoryLocation::Z(idx.parse::<usize>().unwrap()),
            _ => {
                let idx = intermediate_index_mapping
                    .get(&signal_name.to_string())
                    .unwrap();
                MemoryLocation::Intermediate(*idx)
            }
        }
    }

    fn get(&self, memory_location: &MemoryLocation) -> Option<u8> {
        match memory_location {
            MemoryLocation::X(idx) => Some(u8::try_from((self.x >> idx) & 1).unwrap()),
            MemoryLocation::Y(idx) => Some(u8::try_from((self.y >> idx) & 1).unwrap()),
            MemoryLocation::Z(idx) => Some(u8::try_from((self.z >> idx) & 1).unwrap()),
            MemoryLocation::Intermediate(idx) => match self.intermediate_signals.get(*idx) {
                Some(Some(num)) => Some(*num),
                _ => None,
            },
        }
    }

    fn create_gates_from_indexed_gates(&self) -> Vec<Gate> {
        self.indexed_gates
            .iter()
            .map(|ig| {
                let in_1 = self.get_name_name_from_memory_location(&ig.in_1);
                let in_2 = self.get_name_name_from_memory_location(&ig.in_2);
                let out = self.get_name_name_from_memory_location(&ig.out);

                Gate {
                    in_1,
                    in_2,
                    op: ig.op,
                    out,
                }
            })
            .collect_vec()
    }

    fn get_name_name_from_memory_location(&self, memory_location: &MemoryLocation) -> String {
        match memory_location {
            // {:0>width$b}
            MemoryLocation::X(idx) => format!("x{idx:02}"),
            MemoryLocation::Y(idx) => format!("y{idx:02}"),
            MemoryLocation::Z(idx) => format!("z{idx:02}"),
            MemoryLocation::Intermediate(idx) => {
                self.intermediate_signal_names.get(*idx).unwrap().clone()
            }
        }
    }

    fn set(&mut self, memory_location: &MemoryLocation, value: u8) {
        match memory_location {
            MemoryLocation::X(idx) => {
                let mask = 1 << *idx;
                self.x = (self.x & !mask) | (u64::from(value) << idx);
            }
            MemoryLocation::Y(idx) => {
                let mask = 1 << *idx;
                self.y = (self.y & !mask) | (u64::from(value) << idx);
            }
            MemoryLocation::Z(idx) => {
                let mask = 1 << *idx;
                self.z = (self.z & !mask) | (u64::from(value) << idx);
            }
            MemoryLocation::Intermediate(idx) => {
                self.intermediate_signals[*idx] = Some(value);
            }
        }
    }

    fn execute(&mut self, gate: &IndexedGate) -> ExecutionResult {
        let in_1 = self.get(&gate.in_1);
        let in_2 = self.get(&gate.in_2);
        match in_1.zip(in_2) {
            None => ExecutionResult::NotAllInputsAvailable,
            Some((in_1, in_2)) => {
                let result: u8 = gate.op.eval(in_1, in_2);
                self.set(&gate.out, result);
                ExecutionResult::Ok
            }
        }
    }

    fn run_computer(&mut self) {
        let mut operations = VecDeque::from_iter(self.indexed_gates.clone());

        // alternative if cloning is too slow
        // putting all the indices of the indexed_gates into the queue and accessing them like this
        // let foo = &self.indexed_gates[0];
        let mut try_counter = 0;
        while let Some(indexed_gate) = operations.pop_front() {
            // debug!("Try #{try_counter} of {}", operations.len());
            if try_counter > operations.len() {
                // debug!(
                //     "No progress for any gate after {try_counter} of {}",
                //     operations.len()
                // );
                return;
            }
            match self.execute(&indexed_gate) {
                ExecutionResult::NotAllInputsAvailable => {
                    // debug!(
                    //     "Try #{try_counter} of {} not successful - putting gate to the back of the queue", operations.len()
                    // );
                    try_counter += 1;
                    operations.push_back(indexed_gate);
                }
                ExecutionResult::Ok => {
                    // debug!("Try #{try_counter} successful - resetting try_counter");

                    try_counter = 0;
                }
            }
        }
    }
}

enum ExecutionResult {
    Ok,
    NotAllInputsAvailable,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
enum Operator {
    XOR,
    OR,
    AND,
}

impl Operator {
    pub(crate) fn eval(&self, in_1: u8, in_2: u8) -> u8 {
        match self {
            Operator::XOR => BitXor::bitxor(in_1, in_2),
            Operator::OR => BitOr::bitor(in_1, in_2),
            Operator::AND => BitAnd::bitand(in_1, in_2),
        }
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
struct Gate {
    in_1: String,
    in_2: String,
    op: Operator,
    out: String,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
enum MemoryLocation {
    X(usize),
    Y(usize),
    Z(usize),
    Intermediate(usize),
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
struct IndexedGate {
    in_1: MemoryLocation,
    in_2: MemoryLocation,
    op: Operator,
    out: MemoryLocation,
}

fn parse(input: &str) -> IResult<&str, AocComputer> {
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
    //info!("rest: \n{input}");

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
            ),
        ),
    )(input)?;

    Ok((
        input,
        AocComputer::new(
            gates
                .into_iter()
                .map(|((in_1, op, in_2), out)| Gate {
                    in_1: in_1.to_string(),
                    in_2: in_2.to_string(),
                    op,
                    out: out.to_string(),
                })
                .collect_vec(),
            initial_map
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swapping() -> miette::Result<()> {
        let input = r#"
x00: 1
x01: 1
x02: 1
y00: 0
y01: 1
y02: 0

x00 AND y00 -> z00
x01 XOR y01 -> z01
x02 OR y02 -> z02
"#
        .trim();
        let (_, mut computer) = parse(input).unwrap();

        let gates: HashSet<Gate> = computer
            .create_gates_from_indexed_gates()
            .into_iter()
            .collect();

        let original_gates: HashSet<Gate> = computer.gates.iter().cloned().collect();
        assert_eq!(gates, original_gates);

        // Find positions and store just the output values we need
        let (and_gate_pos, and_gate_out) = computer
            .indexed_gates
            .iter()
            .enumerate()
            .find(|(_, ig)| ig.op == Operator::AND)
            .map(|(idx, ig)| (idx, ig.out.clone()))
            .unwrap();

        let (or_gate_pos, or_gate_out) = computer
            .indexed_gates
            .iter()
            .enumerate()
            .find(|(_, ig)| ig.op == Operator::OR)
            .map(|(idx, ig)| (idx, ig.out.clone()))
            .unwrap();

        // Swap
        computer.swap_gate_outputs(and_gate_pos, or_gate_pos);

        let gates: HashSet<Gate> = computer
            .create_gates_from_indexed_gates()
            .into_iter()
            .collect();

        let original_gates: HashSet<Gate> = computer.gates.iter().cloned().collect();
        assert_ne!(gates, original_gates);

        // Check outputs are swapped
        let new_and_gate_out = computer
            .indexed_gates
            .iter()
            .find(|ig| ig.op == Operator::AND)
            .map(|ig| &ig.out)
            .unwrap();

        let new_or_gate_out = computer
            .indexed_gates
            .iter()
            .find(|ig| ig.op == Operator::OR)
            .map(|ig| &ig.out)
            .unwrap();

        assert_eq!(new_and_gate_out, &or_gate_out);
        assert_eq!(new_or_gate_out, &and_gate_out);

        //Swap back
        computer.swap_gate_outputs(and_gate_pos, or_gate_pos);

        let gates: HashSet<Gate> = computer
            .create_gates_from_indexed_gates()
            .into_iter()
            .collect();

        let original_gates: HashSet<Gate> = computer.gates.iter().cloned().collect();
        assert_eq!(gates, original_gates);

        Ok(())
    }

    // broken scenario
    // Checking swap 142-196 for bit #14
    // Running computer with x=22398072246731 and y=759209994931
    // doesn't terminate
    #[test]
    fn debug_swap_endless_compute() {
        let input = include_str!("../input.txt");

        let (_, mut computer) = parse(input).unwrap();
        computer.swap_gate_outputs(142, 196);
        computer.x = 22398072246731;
        computer.y = 759209994931;

        println!("Running computer");
        computer.run_computer();
        println!("Done running computer");
    }
}
