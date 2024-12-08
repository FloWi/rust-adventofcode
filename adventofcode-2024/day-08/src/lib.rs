use glam::IVec2;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use tracing::info;

pub mod part1;
pub mod part2;

fn solve(
    input: &str,
    antinode_finder: impl Fn(&[IVec2], MapDimensions) -> HashSet<IVec2>,
) -> HashSet<IVec2> {
    let (antennas, map_dimensions) = parse(input);

    let grouped_by_frequency = group_antennas_by_frequency(&antennas);
    for (frequency, locations) in &grouped_by_frequency {
        info!("Antenna '{frequency}' locations: {:?}", locations);
    }

    let antinode_locations: HashSet<IVec2> = grouped_by_frequency
        .into_iter()
        .map(|(frequency, antenna_locations)| antinode_finder(&antenna_locations, map_dimensions))
        .fold(HashSet::new(), |mut acc, hash_set| {
            acc.extend(&hash_set);
            acc
        });

    antinode_locations
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
