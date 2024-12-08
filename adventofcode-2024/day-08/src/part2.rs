use glam::IVec2;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::iter::successors;
use tracing::{debug, info};

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (antennas, map_dimensions) = parse(input);

    let grouped_by_frequency = group_antennas_by_frequency(&antennas);
    for (frequency, locations) in &grouped_by_frequency {
        info!("Antenna '{frequency}' locations: {:?}", locations);
    }

    let antinode_locations: HashSet<IVec2> = grouped_by_frequency
        .into_iter()
        .map(|(frequency, locations)| find_antinodes(&locations, map_dimensions))
        .fold(HashSet::new(), |mut acc, hash_set| {
            acc.extend(&hash_set);
            acc
        });

    Ok(antinode_locations.len().to_string())
}

fn group_antennas_by_frequency(antennas: &[AntennaLocation]) -> HashMap<char, Vec<IVec2>> {
    let grouped: HashMap<&char, Vec<&AntennaLocation>> =
        antennas.iter().into_group_map_by(|(loc, freq)| freq);

    let grouped: HashMap<char, Vec<IVec2>> = grouped
        .iter()
        .map(|(key, values)| (**key, values.iter().map(|(loc, _)| *loc).collect_vec()))
        .collect();

    grouped
}

type AntennaLocation = (IVec2, char);
type MapDimensions = (i32, i32);

fn parse(input: &str) -> (Vec<AntennaLocation>, MapDimensions) {
    let lines = input.lines().collect_vec();

    let antenna_locations = input
        .lines()
        .enumerate()
        .flat_map(|(y, line)| {
            line.char_indices().filter_map(move |(x, char)| {
                (char != '.').then_some((IVec2::new(x as i32, y as i32), char))
            })
        })
        .collect_vec();

    let map_dimensions = (lines[0].len() as i32, lines.len() as i32);

    (antenna_locations, map_dimensions)
}

fn find_antinodes(locations: &[IVec2], map_dimensions: MapDimensions) -> HashSet<IVec2> {
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

        let antinodes: HashSet<IVec2> = find_antinodes(antennas.get(&'T').unwrap(), map_dimensions);

        assert_eq!(antinodes, HashSet::from_iter(expected_antinodes));

        Ok(())
    }
}
