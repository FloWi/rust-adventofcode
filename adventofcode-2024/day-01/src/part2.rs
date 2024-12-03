use crate::parse;
use itertools::Itertools;
use miette::miette;
use tracing::info;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, (left, right)) = parse(input).map_err(|e| miette!("parse failed {}", e))?;

    let right_counts = right.iter().counts();
    let similarity_scores = left
        .iter()
        .map(|n| (n, right_counts.get(n).unwrap_or(&0) * (*n as usize)))
        .collect_vec();

    let total_similarity_score: usize = similarity_scores
        .iter()
        .map(|(_, similarity_score)| *similarity_score)
        .sum();

    info!("total_similarity_score: {total_similarity_score}");

    Ok(format!("{total_similarity_score}"))
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
        assert_eq!("31", process(input)?);
        Ok(())
    }
}
