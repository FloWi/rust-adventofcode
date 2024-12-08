use crate::Operator;
use itertools::Itertools;
use miette::miette;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, calibration_equations) =
        crate::parse(input).map_err(|e| miette!("parse failed {}", e))?;

    let result = crate::calc_result(
        calibration_equations,
        vec![Operator::Add, Operator::Multiply],
    );
    Ok(result.to_string())
}

/*
Example 1:
190: 10 19

10 + 19 = 19  // wrong
10 * 19 = 190 // correct

Example 2:
3267: 81 40 27

81 + 40 + 27 = 121 + 27 = 148    // wrong
81 + 40 * 27 = 121 * 27 = 3267   // correct
81 * 40 + 27 = 3240 + 27 = 3267  // correct
81 * 40 * 27 = 3240 * 27 = 87480 // wrong

 */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_all_combinations;

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
        assert_eq!("3749", process(input)?);
        Ok(())
    }

    #[test]
    fn test_foo() {
        let operands = vec![40, 27];
        let operators = vec![Operator::Add, Operator::Multiply];

        let expected_combinations = vec![
            vec![(Operator::Add, 40), (Operator::Add, 27)],
            vec![(Operator::Add, 40), (Operator::Multiply, 27)],
            vec![(Operator::Multiply, 40), (Operator::Add, 27)],
            vec![(Operator::Multiply, 40), (Operator::Multiply, 27)],
        ];

        let actual = get_all_combinations(operands, operators.as_slice());

        assert_eq!(expected_combinations, actual);
    }
}
