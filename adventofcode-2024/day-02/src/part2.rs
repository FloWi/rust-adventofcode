use crate::{parse, validate_report, validate_report_with_problem_dampener};
use itertools::Itertools;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let reports = parse(input)?;

    let valid_report_count = reports.into_iter().filter(validate_report_with_problem_dampener).count();

    Ok(format!("{valid_report_count}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
7 6 4 2 1
1 2 7 8 9
9 7 6 2 1
1 3 2 4 5
8 6 4 4 1
1 3 6 7 9
        "#
            .trim();
        assert_eq!("4", process(input)?);
        Ok(())
    }
}
