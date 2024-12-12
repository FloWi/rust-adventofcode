use itertools::Itertools;
use miette::miette;
use nom::character::complete;
use nom::character::complete::space1;
use nom::multi::separated_list1;
use nom::IResult;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, stones) = parse(input).map_err(|e| miette!("parse failed {}", e))?;

    let stones_collection = (0..25)
        .scan(stones, |stones, _| {
            *stones = apply_rules(stones);
            Some(stones.clone())
        })
        .collect_vec();

    let result = stones_collection.last().unwrap().len();

    Ok(result.to_string())
}

fn parse(input: &str) -> IResult<&str, Vec<u64>> {
    separated_list1(space1, complete::u64)(input)
}

fn apply_rules(stones: &[u64]) -> Vec<u64> {
    stones
        .iter()
        .flat_map(|stone| {
            if *stone == 0 {
                vec![1u64]
            } else if stone
                .checked_ilog10()
                .map(|log10| (log10 + 1) % 2 == 0)
                .unwrap_or(false)
            {
                // If the stone is engraved with a number that has an even number of digits, it is replaced by two stones.
                // The left half of the digits are engraved on the new left stone, and the right half of the digits are engraved on the new right stone.
                // (The new numbers don't keep extra leading zeroes: 1000 would become stones 10 and 0.)

                let number_str = format!("{}", stone);
                let length = number_str.len();
                let (left, right) = number_str.split_at(length / 2);
                vec![left.parse::<u64>().unwrap(), right.parse::<u64>().unwrap()]
            } else {
                vec![stone * 2024u64]
            }
        })
        .collect_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
125 17
        "#
        .trim();
        assert_eq!("55312", process(input)?);
        Ok(())
    }

    #[test]
    fn test_one_blink() -> miette::Result<()> {
        let stones = vec![0, 1, 10, 99, 999];
        let actual = apply_rules(&stones);
        assert_eq!(vec![1, 2024, 1, 0, 9, 9, 2021976], actual);
        Ok(())
    }

    #[test]
    fn test_one_blink_2nd_example_round_1() -> miette::Result<()> {
        let stones = vec![125, 17];
        let actual = apply_rules(&stones);
        assert_eq!(vec![253000, 1, 7], actual);
        Ok(())
    }

    #[test]
    fn test_blink_sequence() -> miette::Result<()> {
        let stones = vec![125, 17];
        let actual = (0..6)
            .scan(stones, |stones, _| {
                *stones = apply_rules(stones);
                Some(stones.clone())
            })
            .collect_vec();

        let actual_str = actual.iter().map(|line| line.iter().join(" ")).join("\n");
        let expected = r#"
253000 1 7
253 0 2024 14168
512072 1 20 24 28676032
512 72 2024 2 0 2 4 2867 6032
1036288 7 2 20 24 4048 1 4048 8096 28 67 60 32
2097446912 14168 4048 2 0 2 4 40 48 2024 40 48 80 96 2 8 6 7 6 0 3 2
        "#
        .trim();
        assert_eq!(expected.to_string(), actual_str);
        Ok(())
    }
}
