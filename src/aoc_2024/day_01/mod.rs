use crate::parsers::streaming_parse_number_pair;
use anyhow::Result;
use itertools::Itertools;
use std::fmt::Debug;

pub(crate) fn part1(input: String) -> Result<String> {
    let (left, right): (Vec<i32>, Vec<i32>) = streaming_parse_number_pair(input.as_str()).unzip();

    let diffs = left
        .iter()
        .sorted()
        .zip(right.iter().sorted())
        .map(|(n1, n2)| (n1, n2, (n1 - n2).abs()))
        .collect_vec();

    let total_distance: i32 = diffs.iter().map(|(_, _, diff)| diff).sum();

    println!("total_distance: {total_distance}");

    Ok(format!("{total_distance}"))
}

pub(crate) fn part2(input: String) -> Result<String> {
    let (left, right): (Vec<i32>, Vec<i32>) = streaming_parse_number_pair(input.as_str()).unzip();

    let right_counts = right.iter().counts();
    let similarity_scores = left
        .iter()
        .map(|n| (n, right_counts.get(n).unwrap_or(&0) * (*n as usize)))
        .collect_vec();

    let total_similarity_score: usize = similarity_scores
        .iter()
        .map(|(_, similarity_score)| *similarity_score)
        .sum();

    println!("total_similarity_score: {total_similarity_score}");

    Ok(format!("{total_similarity_score}"))
}
