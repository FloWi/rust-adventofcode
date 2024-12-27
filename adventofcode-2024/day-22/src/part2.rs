use itertools::Itertools;
use miette::miette;
use nom::character::complete;
use nom::character::complete::line_ending;
use nom::multi::separated_list1;
use nom::{IResult, Parser};
use std::iter::successors;
use std::ops::{BitXor, Index, Rem};

#[tracing::instrument]
pub fn process(_input: &str) -> miette::Result<String> {
    let (_, seed_values) = parse(_input).map_err(|e| miette!("parse failed {}", e))?;

    let result: u64 = 42;
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
            match price_changes {
                None => None,
                Some((((ch_1, ch_2), ch_3), ch_4)) => {
                    Some((initial_price, (ch_1, ch_2, ch_3, ch_4)))
                }
            }
        })
}

fn mix_and_prune(secret: u64, num: u64) -> u64 {
    secret.bitxor(num).rem_euclid(16777216)
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use std::collections::{HashMap, HashSet};
    use std::ops::Not;

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

    #[test]
    fn test_window_functions() {
        let sequence_1 = generate_and_analyze_secrets(1).collect_vec();
        let sequence_2 = generate_and_analyze_secrets(2).collect_vec();
        let sequence_3 = generate_and_analyze_secrets(3).collect_vec();
        let sequence_2024 = generate_and_analyze_secrets(2024).collect_vec();

        let diff_sequence = (-2, 1, -1, 3);

        assert!(sequence_1.contains(&(7, diff_sequence)));
        assert!(sequence_2.contains(&(7, diff_sequence)));
        assert!(sequence_3
            .iter()
            .any(|(_, seq)| seq == &diff_sequence)
            .not());
        assert!(sequence_2024.contains(&(9, diff_sequence)));

        let mut sequence_price_map_1 = HashMap::with_capacity(1000);
        let mut sequence_price_map_2 = HashMap::with_capacity(1000);
        let mut sequence_price_map_3 = HashMap::with_capacity(1000);
        let mut sequence_price_map_2024 = HashMap::with_capacity(1000);

        for (p, changes) in sequence_1.iter() {
            sequence_price_map_1.entry(changes).or_insert(p);
        }
        for (p, changes) in sequence_2.iter() {
            sequence_price_map_2.entry(changes).or_insert(p);
        }
        for (p, changes) in sequence_3.iter() {
            sequence_price_map_3.entry(changes).or_insert(p);
        }
        for (p, changes) in sequence_2024.iter() {
            sequence_price_map_2024.entry(changes).or_insert(p);
        }

        println!("sequence_price_map_1.len(): {}", sequence_price_map_1.len());
        println!("sequence_price_map_2.len(): {}", sequence_price_map_2.len());
        println!("sequence_price_map_3.len(): {}", sequence_price_map_3.len());
        println!(
            "sequence_price_map_2024.len(): {}",
            sequence_price_map_2024.len()
        );

        let all_price_chains: HashSet<&(i8, i8, i8, i8)> = sequence_price_map_1
            .keys()
            .cloned()
            .chain(sequence_price_map_1.keys().cloned())
            .chain(sequence_price_map_3.keys().cloned())
            .chain(sequence_price_map_2024.keys().cloned())
            .collect::<HashSet<_>>();

        let all_maps = [
            sequence_price_map_1,
            sequence_price_map_2,
            sequence_price_map_3,
            sequence_price_map_2024,
        ];

        let best_chains: Vec<((i8, i8, i8, i8), u32)> = all_price_chains
            .iter()
            .map(|price_change_chain| {
                let total: u32 = all_maps
                    .iter()
                    .map(|price_map| **price_map.get(price_change_chain).unwrap_or(&&0u8) as u32)
                    .sum();

                (**price_change_chain, total)
            })
            .sorted_by_key(|(_, total)| *total)
            .rev()
            .collect_vec();

        assert_eq!(best_chains.iter().next().unwrap().1, 23);
    }
}
