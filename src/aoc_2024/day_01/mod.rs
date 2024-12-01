use anyhow::Result;
use itertools::Itertools;
use std::fmt::Debug;
use std::str::FromStr;

pub(crate) fn part1(input: &str) -> Result<String> {
    let numbers: Vec<(i32, i32)> = parse_number_pairs::<i32>(input);

    let (left, right): (Vec<_>, Vec<_>) = numbers.iter().cloned().unzip();

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

pub(crate) fn part2(input: &str) -> Result<String> {
    let numbers: Vec<(i32, i32)> = parse_number_pairs::<i32>(input);

    let (left, right): (Vec<_>, Vec<_>) = numbers.iter().cloned().unzip();

    let right_counts = right.iter().counts();
    let similarity_scores = left
        .iter()
        .map(|n| (n, right_counts.get(n).unwrap_or(&0) * (*n as usize)))
        .collect_vec();

    let total_similarity_score: usize = similarity_scores
        .iter()
        .map(|(_, similarity_score)| *similarity_score)
        .sum();

    // dbg!(&right_counts);
    // dbg!(&similarity_scores);

    println!("total_similarity_score: {total_similarity_score}");

    Ok(format!("{total_similarity_score}"))
}

fn parse_number_pairs<Num>(input: &str) -> Vec<(Num, Num)>
where
    Num: FromStr,
    <Num as FromStr>::Err: Debug,
{
    input
        .lines()
        .map(
            |line| match line.split_whitespace().collect::<Vec<_>>()[..] {
                [a, b] => (
                    a.parse().expect("First must be a number"),
                    b.parse().expect("Second must be a number"),
                ),
                _ => panic!("Input must be exactly two numbers"),
            },
        )
        .collect()
}
