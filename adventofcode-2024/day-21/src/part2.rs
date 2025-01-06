use crate::compute_complexity;
use itertools::Itertools;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let codes = input.trim().lines().collect_vec();

    let complexities = codes
        .iter()
        .cloned()
        .map(|code| compute_complexity(code, 25))
        .collect_vec();

    let result: usize = complexities.iter().sum();

    Ok(result.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
029A
980A
179A
456A
379A
        "#
        .trim();
        assert_eq!("154115708116294", process(input)?);
        Ok(())
    }
}
