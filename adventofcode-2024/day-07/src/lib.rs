use itertools::Itertools;
use nom::bytes::complete::tag;
use nom::character::complete;
use nom::character::complete::{char, line_ending};
use nom::multi::separated_list1;
use nom::sequence::separated_pair;
use nom::IResult;

pub mod part1;
pub mod part2;

fn calc_result(
    calibration_equations: Vec<CalibrationEquation>,
    allowed_operators: Vec<Operator>,
) -> i64 {
    let mut result = 0;

    for calibration_equation in calibration_equations {
        let all_possible_results = eval(calibration_equation.clone(), &allowed_operators);
        if all_possible_results.contains(&calibration_equation.test_value) {
            result += calibration_equation.test_value
        }
    }
    result
}

#[derive(Debug, Clone)]
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

fn create_eval_list(
    calibration_equation: &CalibrationEquation,
    allowed_operators: &[Operator],
) -> (i64, Vec<Vec<(Operator, i64)>>) {
    let first = calibration_equation.operands[0].clone();

    let rest = &calibration_equation.operands[1..];

    let all_operators = allowed_operators;

    let operations = get_all_combinations(rest.to_vec(), all_operators);

    (first, operations)
}

fn get_all_combinations(operands: Vec<i64>, operators: &[Operator]) -> Vec<Vec<(Operator, i64)>> {
    // Create a vector of operator references n times (where n is the number of operands)
    let operator_sets = std::iter::repeat(operators.iter())
        .take(operands.len())
        .collect::<Vec<_>>();

    // Get all possible combinations of operators
    let operations = operator_sets
        .into_iter()
        .multi_cartesian_product()
        .map(|ops| {
            ops.into_iter()
                .zip(operands.iter())
                .map(|(op, num)| (op.clone(), *num))
                .collect()
        })
        .collect();

    operations
}

fn eval(calibration_equation: CalibrationEquation, allowed_operators: &[Operator]) -> Vec<i64> {
    let (first, permutations) = create_eval_list(&calibration_equation, allowed_operators);

    permutations
        .iter()
        .map(|permutation| {
            let result = permutation.iter().fold(first, |acc, (operator, operand)| {
                operator.perform(acc, *operand)
            });
            result
        })
        .collect_vec()
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
enum Operator {
    Add,
    Multiply,
    Concat,
}

impl Operator {
    pub(crate) fn perform(&self, p0: i64, p1: i64) -> i64 {
        match self {
            Operator::Add => p0 + p1,
            Operator::Multiply => p0 * p1,
            Operator::Concat => format!("{p0}{p1}").parse::<i64>().unwrap(),
        }
    }
}
