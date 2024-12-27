use miette::miette;
use nom::character::complete;
use nom::character::complete::line_ending;
use nom::multi::separated_list1;
use nom::IResult;
use std::iter::successors;
use std::ops::{BitXor, Rem};

#[tracing::instrument]
pub fn process(_input: &str) -> miette::Result<String> {
    let (_, seed_values) = parse(_input).map_err(|e| miette!("parse failed {}", e))?;

    let result: u64 = seed_values
        .into_iter()
        .map(|seed| generate_secrets(seed).skip(1999).next().unwrap())
        .sum();
    Ok(result.to_string())
}

fn parse(input: &str) -> IResult<&str, Vec<u64>> {
    separated_list1(line_ending, complete::u64)(input)
}

fn generate_secrets(initial: u64) -> impl Iterator<Item = u64> {
    successors(Some(initial), |curr| {
        let next_value_1 = mix_and_prune(*curr, curr * 64);
        let next_value_2 = mix_and_prune(next_value_1, next_value_1 / 32);
        let next_value_3 = mix_and_prune(next_value_2, next_value_2 * 2048);
        Some(next_value_3)
    })
    .skip(1)
}

fn mix_and_prune(secret: u64, num: u64) -> u64 {
    secret.bitxor(num).rem_euclid(16777216)
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
1
10
100
2024
        "#
        .trim();
        assert_eq!("37327623", process(input)?);
        Ok(())
    }

    #[test]
    fn test_generate_sequence() -> miette::Result<()> {
        let initial_secret_number = 123;
        let expected_next_10_secrets = vec![
            15887950, 16495136, 527345, 704524, 1553684, 12683156, 11100544, 12249484, 7753432,
            5908254,
        ];

        let actual = generate_secrets(123).take(10).collect_vec();

        assert_eq!(actual, expected_next_10_secrets);

        Ok(())
    }
}
