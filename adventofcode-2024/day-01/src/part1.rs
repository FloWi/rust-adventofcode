use itertools::Itertools;
use miette::miette;
use tracing::info;
use crate::parse;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, (left, right)) = parse(input)
        .map_err(|e| miette!("parse failed {}", e))?;

    let diffs = left
        .iter()
        .sorted()
        .zip(right.iter().sorted())
        .map(|(n1, n2)| (n1, n2, (n1 - n2).abs()))
        .collect_vec();

    let total_distance: i32 = diffs.iter().map(|(_, _, diff)| diff).sum();

    info!("total_distance: {total_distance}");

    Ok(format!("{total_distance}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
3   4
4   3
2   5
1   3
3   9
3   3
        "#
        .trim();
        assert_eq!("11", process(input)?);
        Ok(())
    }
}
