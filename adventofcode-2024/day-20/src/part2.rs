use crate::{find_path, parse};
use glam::IVec2;
use itertools::Itertools;
use miette::miette;
use nom::character::complete;
use nom::IResult;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    process_parameterized(input, 100)
}

fn parse_args(args: &str) -> IResult<&str, u32> {
    let (remaining, value) = complete::u32(args)?;

    Ok((remaining, value))
}

pub fn process_with_args(input: &str, args: &str) -> miette::Result<String> {
    let (_, min_savings_limit) = parse_args(args).map_err(|e| miette!("arg-parse failed {}", e))?;

    Ok(process_parameterized(input, min_savings_limit)?)
}

#[tracing::instrument]
pub fn process_parameterized(input: &str, min_savings_limit: u32) -> miette::Result<String> {
    let racetrack = parse(input);

    let (path, cost) =
        find_path(&racetrack.walls, &racetrack.start, &racetrack.end).expect("path to be found");

    let result = find_number_of_cheats(path, cost, min_savings_limit);

    Ok(result.to_string())
}

fn find_number_of_cheats(
    path: Vec<IVec2>,
    complete_path_cost: u32,
    min_savings_limit: u32,
) -> usize {
    let max_cheat_length = 20;

    // - check every node in the path
    //   - from there - check every succeeding node and check the manhattan distance
    //   - if this distance is lower than the original costs at this point, we've found a cheat

    let result = path
        .iter()
        .enumerate()
        .tuple_combinations()
        .filter_map(|((cost_n1, pos_n1), (cost_n2, pos_n2))| {
            let manhattan_distance = (pos_n2 - pos_n1).abs().element_sum() as u32;

            if manhattan_distance > max_cheat_length {
                None
            } else {
                let original_cost_n2_to_destination = complete_path_cost - cost_n2 as u32;
                let complete_costs_using_cheat =
                    cost_n1 as u32 + manhattan_distance + original_cost_n2_to_destination;

                let savings = complete_path_cost - complete_costs_using_cheat;

                (savings >= min_savings_limit).then_some(complete_costs_using_cheat)
            }
        })
        .count();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
###############
#...#...#.....#
#.#.#.#.#.###.#
#S#...#.#.#...#
#######.#.#.###
#######.#.#...#
#######.#.###.#
###..E#...#...#
###.#######.###
#...###...#...#
#.#####.#.###.#
#.#...#.#.#...#
#.#.#.#.#.#.###
#...#...#...###
###############
        "#
        .trim();

        // There are 32 cheats that save 50 picoseconds.
        // There are 31 cheats that save 52 picoseconds.
        // There are 29 cheats that save 54 picoseconds.
        // There are 39 cheats that save 56 picoseconds.
        // There are 25 cheats that save 58 picoseconds.
        // There are 23 cheats that save 60 picoseconds.
        // There are 20 cheats that save 62 picoseconds.
        // There are 19 cheats that save 64 picoseconds.
        // There are 12 cheats that save 66 picoseconds.
        // There are 14 cheats that save 68 picoseconds.
        // There are 12 cheats that save 70 picoseconds.
        // There are 22 cheats that save 72 picoseconds.
        // There are 4 cheats that save 74 picoseconds.
        // There are 3 cheats that save 76 picoseconds.

        let expected_number_of_cheats: i32 = [32, 31, 29, 39, 25, 23, 20, 19, 12, 14, 12, 22, 4, 3]
            .iter()
            .sum();

        // I thought this was a valid testcase. My solution created the correct answer for part 2 but not for this testcase.
        assert_eq!(
            expected_number_of_cheats.to_string(),
            process_parameterized(input, 50)?
        );
        Ok(())
    }

    #[test]
    fn pathfinding() {
        let input = r#"
###############
#...#...#.....#
#.#.#.#.#.###.#
#S#...#.#.#...#
#######.#.#.###
#######.#.#...#
#######.#.###.#
###..E#...#...#
###.#######.###
#...###...#...#
#.#####.#.###.#
#.#...#.#.#...#
#.#.#.#.#.#.###
#...#...#...###
###############
        "#
        .trim();

        let racetrack = parse(input);

        let (_path, cost) = find_path(&racetrack.walls, &racetrack.start, &racetrack.end)
            .expect("path to be found");

        assert_eq!(cost, 84);
    }
}
