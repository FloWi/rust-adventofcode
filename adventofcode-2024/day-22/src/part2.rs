use itertools::Itertools;
use miette::miette;
use nom::character::complete;
use nom::character::complete::line_ending;
use nom::multi::separated_list1;
use nom::IResult;
use std::collections::HashMap;
use std::iter::successors;
use std::ops::BitXor;

#[tracing::instrument]
pub fn process(_input: &str) -> miette::Result<String> {
    let (_, seed_values) = parse(_input).map_err(|e| miette!("parse failed {}", e))?;

    let result: u64 = find_best_purchase_diff_sequence(seed_values);
    Ok(result.to_string())
}

fn parse(input: &str) -> IResult<&str, Vec<u64>> {
    separated_list1(line_ending, complete::u64)(input)
}

fn generate_secrets(initial: u64) -> impl Iterator<Item = (u64, u8, Option<i8>)> {
    successors(
        Some((initial, initial.rem_euclid(10) as u8, None)),
        |(curr, prev_ones_digit, _)| {
            let next_value_1 = mix_and_prune(*curr, curr * 64);
            let next_value_2 = mix_and_prune(next_value_1, next_value_1 / 32);
            let next_value_3 = mix_and_prune(next_value_2, next_value_2 * 2048);
            let next_ones_digit = next_value_3.rem_euclid(10);
            Some((
                next_value_3,
                next_ones_digit as u8,
                Some(next_ones_digit as i8 - *prev_ones_digit as i8),
            ))
        },
    )
}

fn generate_and_analyze_secrets(initial: u64) -> impl Iterator<Item = (u8, (i8, i8, i8, i8))> {
    generate_secrets(initial)
        .take(2000)
        .tuple_windows()
        .filter_map(|(tup_1, tup_2, tup_3, tup_4)| {
            let initial_price = tup_4.1;
            let price_changes = tup_1.2.zip(tup_2.2).zip(tup_3.2).zip(tup_4.2);
            price_changes
                .map(|(((ch_1, ch_2), ch_3), ch_4)| (initial_price, (ch_1, ch_2, ch_3, ch_4)))
        })
}

fn mix_and_prune(secret: u64, num: u64) -> u64 {
    secret.bitxor(num).rem_euclid(16777216)
}

fn find_best_purchase_diff_sequence(seeds: Vec<u64>) -> u64 {
    let summary_map_capacity = seeds.len() * 20; //rough guesstimate - for the real input (2500 seeds, the final map had 40951 entries)

    use rayon::prelude::*;

    let summary_map: HashMap<_, _> = seeds
        .into_par_iter()
        .map(|seed| {
            generate_and_analyze_secrets(seed).fold(
                HashMap::with_capacity(2000),
                |mut acc, (price, changes)| {
                    // only the first occurrence of the diff_chain is relevant
                    acc.entry(changes).or_insert(price as u64);
                    acc
                },
            )
        })
        .reduce(
            || HashMap::with_capacity(summary_map_capacity),
            |mut outer_acc, per_seed_acc| {
                per_seed_acc.into_iter().for_each(|(changes, p)| {
                    outer_acc
                        .entry(changes)
                        .and_modify(|counter| *counter += p)
                        .or_insert(p);
                });
                outer_acc
            },
        );

    summary_map
        .into_iter()
        .sorted_by_key(|tup| tup.1)
        .next_back()
        .unwrap()
        .1
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
1
2
3
2024
        "#
        .trim();
        assert_eq!("23", process(input)?);
        Ok(())
    }

    #[test]
    fn test_generate_sequence() -> miette::Result<()> {
        let expected_next_10_secrets: Vec<(u64, u8, Option<i8>)> = vec![
            (123, 3, None),
            (15887950, 0, Some(-3)),
            (16495136, 6, Some(6)),
            (527345, 5, Some(-1)),
            (704524, 4, Some(-1)),
            (1553684, 4, Some(0)),
            (12683156, 6, Some(2)),
            (11100544, 4, Some(-2)),
            (12249484, 4, Some(0)),
            (7753432, 2, Some(-2)),
        ];

        let actual = generate_secrets(123).take(10).collect_vec();

        assert_eq!(actual, expected_next_10_secrets);

        Ok(())
    }
}
