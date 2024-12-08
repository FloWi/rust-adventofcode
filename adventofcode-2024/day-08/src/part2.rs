use crate::{solve, MapDimensions};
use glam::IVec2;
use itertools::Itertools;
use std::collections::HashSet;
use std::iter::successors;
use tracing::debug;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let antinode_locations = solve(input, find_antinodes_part_2);

    Ok(antinode_locations.len().to_string())
}

fn find_antinodes_part_2(locations: &[IVec2], map_dimensions: MapDimensions) -> HashSet<IVec2> {
    let antinode_locations = locations
        .iter()
        .tuple_combinations()
        .flat_map(|(a, b)| {
            let distance = a - b;
            debug!("a: {a}, b: {b}: Distance: {distance}");
            let iter_1 =
                successors(Some(*a), move |loc_a| Some(*loc_a + distance)).take_while(|loc| {
                    loc.x >= 0 && loc.x < map_dimensions.0 && loc.y >= 0 && loc.y < map_dimensions.1
                });
            let iter_2 =
                successors(Some(*b), move |loc_b| Some(*loc_b - distance)).take_while(|loc| {
                    loc.x >= 0 && loc.x < map_dimensions.0 && loc.y >= 0 && loc.y < map_dimensions.1
                });

            iter_1.chain(iter_2)
        })
        .collect_vec();

    antinode_locations.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{group_antennas_by_frequency, parse};
    use std::collections::HashSet;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
............
........0...
.....0......
.......0....
....0.......
......A.....
............
............
........A...
.........A..
............
............
        "#
        .trim();
        assert_eq!("34", process(input)?);
        Ok(())
    }

    #[test]
    fn test_example_1() -> miette::Result<()> {
        let input = r#"
T....#....
...T......
.T....#...
.........#
..#.......
..........
...#......
..........
....#.....
..........
        "#
        .trim();

        let (map_locations, map_dimensions) = parse(input);

        let mut grouped_by_frequency = group_antennas_by_frequency(&map_locations);

        // antinodes are at the '#' locations and also at the initial antenna-locations
        let mut expected_antinodes = grouped_by_frequency.remove(&'#').unwrap();
        expected_antinodes.extend(grouped_by_frequency.get(&'T').unwrap());

        let antennas = grouped_by_frequency;
        dbg!(&expected_antinodes);
        dbg!(&antennas);

        let antinodes: HashSet<IVec2> =
            find_antinodes_part_2(antennas.get(&'T').unwrap(), map_dimensions);

        assert_eq!(antinodes, HashSet::from_iter(expected_antinodes));

        Ok(())
    }
}
