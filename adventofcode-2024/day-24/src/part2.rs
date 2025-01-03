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

pub fn process(input: &str) -> miette::Result<String> {
    let (_, mut aoc_computer) = parse(input).map_err(|e| miette!("parse failed {}", e))?;

    aoc_computer.run_computer();

    Ok(aoc_computer.z.to_string())
}

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
}

impl AocComputer {
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
        while let Some(indexed_gate) = operations.pop_front() {
            match self.execute(&indexed_gate) {
                ExecutionResult::NotAllInputsAvailable => {
                    operations.push_back(indexed_gate);
                }
                ExecutionResult::Ok => {}
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
