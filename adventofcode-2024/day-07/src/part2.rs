use crate::Operator;
use itertools::Itertools;
use miette::miette;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, calibration_equations) =
        crate::parse(input).map_err(|e| miette!("parse failed {}", e))?;

    let result = crate::calc_result(
        calibration_equations,
        vec![Operator::Add, Operator::Multiply, Operator::Concat],
    );
    Ok(result.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!("11387", process(input)?);
        Ok(())
    }
}
