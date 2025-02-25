use crate::{find_path, parse, Racetrack, NEIGHBORS};
use glam::IVec2;
use itertools::Itertools;
use miette::miette;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete;
use nom::combinator::{map, value};
use nom::sequence::{delimited, separated_pair};
use nom::IResult;
use std::collections::HashMap;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    process_parameterized(input, Some(100))
}

fn parse_args(args: &str) -> IResult<&str, Option<u32>> {
    let (remaining, maybe_value) = alt((
        value(None, tag("None")),
        map(
            delimited(tag("Some("), complete::u32, tag(")")),
            |some_value: u32| Some(some_value),
        ),
    ))(args)?;

    Ok((remaining, maybe_value))
}

pub fn process_with_args(input: &str, args: &str) -> miette::Result<String> {
    let (_, maybe_min_savings_limit) =
        parse_args(args).map_err(|e| miette!("arg-parse failed {}", e))?;

    Ok(process_parameterized(input, maybe_min_savings_limit)?)
}

#[tracing::instrument]
pub fn process_parameterized(
    input: &str,
    min_savings_limit: Option<u32>,
) -> miette::Result<String> {
    let racetrack = parse(input);

    let (path, cost) =
        find_path(&racetrack.walls, &racetrack.start, &racetrack.end).expect("path to be found");

    let savings_map = find_number_of_cheats(path, &racetrack);

    let relevant_entries = match min_savings_limit {
        None => savings_map,
        Some(lower_bound) => savings_map
            .into_iter()
            .filter(|(savings, _)| *savings >= lower_bound as i32)
            .collect(),
    };

    let result = relevant_entries.values().sum::<u32>();

    Ok(result.to_string())
}

fn find_number_of_cheats(path: Vec<IVec2>, racetrack: &Racetrack) -> HashMap<i32, u32> {
    let visited_tiles: HashMap<IVec2, i32> = path
        .iter()
        .enumerate()
        .map(|(idx, loc)| (*loc, idx as i32))
        .collect();

    // check the wall-neighbors of all wall-tiles (and their neighbors?) to see if they lead to a path-tile again.
    // might work, because the problem statement said: "Because there is only a single path from the start to the end"

    let mut savings_map: HashMap<i32, u32> = HashMap::new();

    for (curr, next) in path.iter().tuple_windows() {
        for wall_neighbor_1 in NEIGHBORS.map(|offset| curr + offset) {
            if racetrack.walls.contains(&wall_neighbor_1) {
                for neighbor_2 in NEIGHBORS.map(|offset| wall_neighbor_1 + offset) {
                    if !racetrack.walls.contains(&neighbor_2)
                        && visited_tiles.contains_key(&neighbor_2)
                        && &neighbor_2 != curr
                        && &neighbor_2 != next
                    {
                        let cost_current = visited_tiles[curr];
                        let cost_neighbor_2_original = visited_tiles[&neighbor_2];

                        let savings = cost_neighbor_2_original - cost_current - 2;
                        if savings > 0 {
                            savings_map
                                .entry(savings)
                                .and_modify(|e| *e += 1u32)
                                .or_insert(1u32);
                            //println!("shortcut from {curr} (cost {cost_current}) to {neighbor_2} (cost original {cost_neighbor_2_original}) saves {savings} by removing wall {wall_neighbor_1}.")
                        }
                    }
                }
            }
        }
    }

    savings_map
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

        // There are 14 cheats that save 2 picoseconds.
        // There are 14 cheats that save 4 picoseconds.
        // There are 2 cheats that save 6 picoseconds.
        // There are 4 cheats that save 8 picoseconds.
        // There are 2 cheats that save 10 picoseconds.
        // There are 3 cheats that save 12 picoseconds.
        // There is one cheat that saves 20 picoseconds.
        // There is one cheat that saves 36 picoseconds.
        // There is one cheat that saves 38 picoseconds.
        // There is one cheat that saves 40 picoseconds.
        // There is one cheat that saves 64 picoseconds.

        let expected_number_of_cheats: i32 = [14, 14, 2, 4, 2, 3, 1, 1, 1, 1, 1].iter().sum();
        assert_eq!(
            process_parameterized(input, None)?,
            expected_number_of_cheats.to_string()
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
