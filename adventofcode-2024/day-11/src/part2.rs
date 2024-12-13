use itertools::Itertools;
use miette::miette;
use nom::character::complete;
use nom::character::complete::space1;
use nom::multi::separated_list1;
use nom::IResult;
use std::collections::HashMap;
use std::time::Instant;
use tracing::debug;

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, stones) = parse(input).map_err(|e| miette!("parse failed {}", e))?;

    let stones_map: HashMap<u64, usize> = stones.into_iter().counts();

    let stones_collection = (0..75)
        .scan(stones_map, |stones_map, idx| {
            *stones_map = apply_rules(stones_map, idx);
            Some(stones_map.clone())
        })
        .collect_vec();

    let final_stones = stones_collection.last().unwrap();
    debug!(
        "found {} different stones in final step",
        final_stones.len()
    );
    let result = final_stones.values().sum::<usize>();

    Ok(result.to_string())
}

fn parse(input: &str) -> IResult<&str, Vec<u64>> {
    separated_list1(space1, complete::u64)(input)
}

#[tracing::instrument(skip(stones), fields(idx = idx))]
fn apply_rules(stones: &HashMap<u64, usize>, idx: i32) -> HashMap<u64, usize> {
    let start = Instant::now();
    let num_initial_stones: usize = stones.values().sum();

    let mut new_stones = stones.clone();

    for (stone, qty) in stones {
        new_stones
            .entry(*stone)
            .and_modify(|counter| *counter -= *qty);
        if *stone == 0 {
            new_stones
                .entry(1u64)
                .and_modify(|counter| *counter += *qty)
                .or_insert(*qty);
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
            new_stones
                .entry(left.parse::<u64>().unwrap())
                .and_modify(|counter| *counter += *qty)
                .or_insert(*qty);
            new_stones
                .entry(right.parse::<u64>().unwrap())
                .and_modify(|counter| *counter += *qty)
                .or_insert(*qty);
        } else {
            new_stones
                .entry(stone * 2024u64)
                .and_modify(|counter| *counter += *qty)
                .or_insert(*qty);
        }
    }

    debug!(
        target: "applying rules",
        initial_stones = num_initial_stones,
        new_stones = new_stones.values().sum::<usize>(),
        duration_ms = start.elapsed().as_millis()
    );
    new_stones
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_process_my_own_result() -> miette::Result<()> {
        let input = r#"
125 17
        "#
        .trim();
        // no expected result given for part 2
        assert_eq!("65601038650482", process(input)?);
        Ok(())
    }

    fn sorted_occurrences_string(occurrences: HashMap<u64, usize>) -> String {
        occurrences
            .iter()
            .filter(|(_, &v)| v > 0)
            .sorted_by_key(|(k, v)| *k)
            .map(|(k, v)| format!("{k}: {v}x"))
            .join("\n")
    }

    #[test]
    fn test_one_blink() -> miette::Result<()> {
        let stones: HashMap<u64, usize> =
            [0, 1, 10, 99, 999].iter().map(|i| *i as u64).counts();
        let actual = apply_rules(&stones, 0);
        let expected = [1, 2024, 1, 0, 9, 9, 2021976]
            .iter()
            .map(|i| *i as u64)
            .counts();
        assert_eq!(
            sorted_occurrences_string(actual),
            sorted_occurrences_string(expected)
        );
        Ok(())
    }
    //
    //     #[test]
    //     fn test_one_blink_2nd_example_round_1() -> miette::Result<()> {
    //         let stones = vec![125, 17];
    //         let actual = apply_rules(&stones, 0);
    //         assert_eq!(vec![253000, 1, 7], actual);
    //         Ok(())
    //     }
    //
    //     #[test]
    //     fn test_blink_sequence() -> miette::Result<()> {
    //         let stones = vec![125, 17];
    //         let actual = (0..6)
    //             .scan(stones, |(stones), idx| {
    //                 *stones = apply_rules(stones, idx);
    //                 Some(stones.clone())
    //             })
    //             .collect_vec();
    //
    //         let actual_str = actual
    //             .iter()
    //             .map(|line| line.into_iter().join(" "))
    //             .join("\n");
    //         let expected = r#"
    // 253000 1 7
    // 253 0 2024 14168
    // 512072 1 20 24 28676032
    // 512 72 2024 2 0 2 4 2867 6032
    // 1036288 7 2 20 24 4048 1 4048 8096 28 67 60 32
    // 2097446912 14168 4048 2 0 2 4 40 48 2024 40 48 80 96 2 8 6 7 6 0 3 2
    //         "#
    //         .trim();
    //
    //         assert_eq!(expected.to_string(), actual_str);
    //         Ok(())
    //     }
}
