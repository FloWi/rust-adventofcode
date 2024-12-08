use crate::{solve, MapDimensions};
use glam::IVec2;
use itertools::Itertools;
use std::collections::HashSet;
use tracing::debug;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let antinode_locations = solve(input, find_antinodes);

    Ok(antinode_locations.len().to_string())
}

fn find_antinodes(locations: &[IVec2], map_dimensions: MapDimensions) -> HashSet<IVec2> {
    let antinode_locations = locations
        .iter()
        .tuple_combinations()
        .flat_map(|(a, b)| {
            let distance = a - b;
            debug!("a: {a}, b: {b}: Distance: {distance}");
            let location_1 = a + distance;
            let location_2 = b - distance;
            debug!("  Location_1: {location_1}");
            debug!("  Location_2: {location_2}");
            vec![location_1, location_2]
        })
        .filter(|loc| {
            loc.x >= 0 && loc.x < map_dimensions.0 && loc.y >= 0 && loc.y < map_dimensions.1
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
        assert_eq!("14", process(input)?);
        Ok(())
    }

    #[test]
    fn test_example_2() -> miette::Result<()> {
        let input = r#"
..........
...#......
..........
....a.....
..........
.....a....
..........
......#...
..........
..........
        "#
        .trim();

        let (map_locations, map_dimensions) = parse(input);

        let mut grouped_by_frequency = group_antennas_by_frequency(&map_locations);
        let expected_antinodes = grouped_by_frequency.remove(&'#').unwrap();

        let antennas = grouped_by_frequency;
        dbg!(&expected_antinodes);
        dbg!(&antennas);

        let antinodes: HashSet<IVec2> = find_antinodes(antennas.get(&'a').unwrap(), map_dimensions);

        assert_eq!(antinodes, HashSet::from_iter(expected_antinodes));

        Ok(())
    }

    #[test]
    fn test_example_3() -> miette::Result<()> {
        let input = r#"
..........
...#......
#.........
....a.....
........a.
.....a....
..#.......
......#...
..........
..........
        "#
        .trim();

        let (map_locations, map_dimensions) = parse(input);

        let mut grouped_by_frequency = group_antennas_by_frequency(&map_locations);
        let expected_antinodes = grouped_by_frequency.remove(&'#').unwrap();

        let antennas = grouped_by_frequency;
        dbg!(&expected_antinodes);
        dbg!(&antennas);

        let antinodes: HashSet<IVec2> = find_antinodes(antennas.get(&'a').unwrap(), map_dimensions);

        assert_eq!(antinodes, HashSet::from_iter(expected_antinodes));

        Ok(())
    }
}
