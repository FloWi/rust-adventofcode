use glam::IVec2;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use tracing::debug;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let parsed_tiles: HashMap<IVec2, char> = parse(input);
    //dbg!(&parsed_tiles);
    let all_areas: HashMap<char, Vec<HashSet<IVec2>>> = find_areas(parsed_tiles.clone());

    let scores = score_areas(&all_areas, parsed_tiles.clone());
    let updated_scores: Vec<ScoredArea> = add_edge_score_to_areas(&scores);

    //dbg!(&updated_scores);

    let result: usize = updated_scores
        .iter()
        .map(|scored_area| scored_area.score)
        .sum();

    Ok(result.to_string())
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
enum EdgeDir {
    North,
    East,
    South,
    West,
}

const EDGE_OFFSETS: [(EdgeDir, IVec2); 4] = [
    (EdgeDir::North, IVec2::NEG_Y),
    (EdgeDir::East, IVec2::X),
    (EdgeDir::South, IVec2::Y),
    (EdgeDir::West, IVec2::NEG_X),
];

fn add_edge_score_to_areas(scored_areas: &Vec<ScoredArea>) -> Vec<ScoredArea> {
    scored_areas
        .clone()
        .into_iter()
        .map(|scored_area| {
            let outer_edges = scored_area
                .area
                .iter()
                .flat_map(|loc| {
                    EDGE_OFFSETS.into_iter().filter_map({
                        // very ugly, but borrow-checker was more stubborn than me
                        let area = scored_area.area.clone();
                        move |(dir, offset)| {
                            // we can add an outside edge if the neighbor in that direction is non-existent
                            let neighbor_location = loc + offset;
                            if area.contains(&neighbor_location) {
                                // inner edge - don't add
                                None
                            } else {
                                // outer edge
                                Some((*loc, dir))
                            }
                        }
                    })
                })
                .collect_vec();

            let better_edge_groups: Vec<Vec<(IVec2, EdgeDir)>> =
                better_combine_edges_into_adjacent_edge_groups(&outer_edges);

            let perimeter = better_edge_groups.len();
            let score = scored_area.size * perimeter;

            ScoredArea {
                merged_edges: better_edge_groups.clone(),
                perimeter,
                score,
                ..scored_area
            }
        })
        .collect_vec()
}

fn better_combine_edges_into_adjacent_edge_groups(
    outer_edges: &Vec<(IVec2, EdgeDir)>,
) -> Vec<Vec<(IVec2, EdgeDir)>> {
    /*
    outer edges for 'A': [2, 0]N, [2, 0]S, [1, 0]N, [1, 0]S, [0, 0]N, [0, 0]S, [0, 0]W, [3, 0]N, [3, 0]E, [3, 0]S

    E x=3 [3, 0]
    W x=0 [0, 0]
    N y=0 [0, 0]
    N y=0 [1, 0]
    N y=0 [2, 0]
    N y=0 [3, 0]
    S y=0 [0, 0]
    S y=0 [1, 0]
    S y=0 [2, 0]
    S y=0 [3, 0]
         */

    let horizontal_edge_group_candidates = outer_edges
        .iter()
        .filter(|(_, dir)| match dir {
            EdgeDir::North | EdgeDir::South => true,
            EdgeDir::East | EdgeDir::West => false,
        })
        .into_group_map_by(|(loc, dir)| (loc.y, dir));

    let vertical_edge_group_candidates = outer_edges
        .iter()
        .filter(|(_, dir)| match dir {
            EdgeDir::East | EdgeDir::West => true,
            EdgeDir::North | EdgeDir::South => false,
        })
        .into_group_map_by(|(loc, dir)| (loc.x, dir));

    let horizontal_edge_groups = horizontal_edge_group_candidates
        .into_iter()
        .flat_map(|((y, dir), candidates)| {
            let xs = candidates
                .iter()
                .map(|(loc, _)| loc.x)
                .sorted()
                .collect_vec();
            let consecutive_ranges = find_consecutive_ranges(&xs);

            consecutive_ranges
                .iter()
                .map(|range| {
                    range
                        .iter()
                        .map(|x| (IVec2::new(*x, y), dir.clone()))
                        .collect_vec()
                })
                .collect_vec()
        })
        .collect_vec();

    let vertical_edge_groups = vertical_edge_group_candidates
        .into_iter()
        .flat_map(|((x, dir), candidates)| {
            let ys = candidates
                .iter()
                .map(|(loc, _)| loc.y)
                .sorted()
                .collect_vec();
            let consecutive_ranges = find_consecutive_ranges(&ys);

            consecutive_ranges
                .iter()
                .map(|range| {
                    range
                        .iter()
                        .map(|y| (IVec2::new(x, *y), dir.clone()))
                        .collect_vec()
                })
                .collect_vec()
        })
        .collect_vec();

    [&horizontal_edge_groups[..], &vertical_edge_groups[..]].concat()
}

// Alternative approach using windows to make the logic more explicit
fn find_consecutive_ranges(numbers: &[i32]) -> Vec<Vec<i32>> {
    if numbers.is_empty() {
        return vec![];
    }

    let splits = numbers
        .windows(2)
        .enumerate()
        // Find positions where the difference isn't 1
        .filter(|(_, w)| w[1] - w[0] != 1)
        .map(|(i, _)| i + 1)
        .collect::<Vec<_>>();

    // Use the split positions to create our ranges
    std::iter::once(0)
        .chain(splits)
        .chain(std::iter::once(numbers.len()))
        .collect::<Vec<_>>()
        .windows(2)
        .map(|w| numbers[w[0]..w[1]].to_vec())
        .collect()
}

#[derive(Debug, Clone)]
struct ScoredArea {
    area: HashSet<IVec2>,
    label: char,
    size: usize,
    perimeter: usize,
    score: usize,
    merged_edges: Vec<Vec<(IVec2, EdgeDir)>>,
}

fn score_areas(
    areas: &HashMap<char, Vec<HashSet<IVec2>>>,
    tiles: HashMap<IVec2, char>,
) -> Vec<ScoredArea> {
    let mut scored_areas = Vec::new();

    for (label, areas_of_label) in areas {
        for area in areas_of_label {
            let perimeter_length = area
                .iter()
                .map(|&pos| {
                    NEIGHBOR_DIRECTIONS
                        .iter()
                        .map(|dir| {
                            let adjacent_location = *dir + pos;
                            match tiles.get(&adjacent_location) {
                                None => {
                                    // no neighbor
                                    1
                                }
                                Some(neighbor_label) => {
                                    if neighbor_label == label {
                                        0
                                    } else {
                                        1
                                    }
                                }
                            }
                        })
                        .sum::<usize>()
                })
                .sum::<usize>();
            scored_areas.push(ScoredArea {
                area: area.clone(),
                label: *label,
                size: area.len(),
                perimeter: perimeter_length,
                score: area.len() * perimeter_length,

                merged_edges: vec![],
            });
        }
    }

    scored_areas
}

fn find_areas(tiles: HashMap<IVec2, char>) -> HashMap<char, Vec<HashSet<IVec2>>> {
    let char_locations: HashMap<char, HashSet<IVec2>> = tiles
        .into_iter()
        .map(|(loc, char)| (char, loc))
        .into_group_map()
        .into_iter()
        .map(|(char, locations)| (char, HashSet::from_iter(locations)))
        .collect();

    let mut result = HashMap::new();

    for (char, locations) in char_locations {
        debug!(
            "\n\nfind_areas: processing '{char}' with {} locations",
            locations.len()
        );
        let mut areas: Vec<HashSet<IVec2>> = Vec::new();

        for loc in &locations {
            let neighbors: HashSet<IVec2> = NEIGHBOR_DIRECTIONS
                .map(|dir| loc + dir)
                .into_iter()
                .filter(|neighbor| locations.contains(neighbor))
                .collect();
            let matching_areas = areas
                .iter()
                .filter(|&area| {
                    area.contains(loc) || area.iter().any(|node| neighbors.contains(node))
                })
                .cloned()
                .collect_vec();

            debug!("Evaluating loc {loc}. Found {} neighbor(s). Loc and or neighbors are contained in {} area(s)", neighbors.len(), matching_areas.len());

            if matching_areas.is_empty() {
                let mut new_area = HashSet::new();
                new_area.insert(*loc);
                for neighbor in neighbors.iter() {
                    new_area.insert(*neighbor);
                }
                debug!("No matching area found. Creating new one with location and all relevant neighbors: {new_area:?}");
                areas.push(new_area);
            } else {
                // we can now merge the areas together, since we found connecting tile(s) between all of them.
                let mut new_area = matching_areas.iter().fold(HashSet::new(), |acc, curr| {
                    acc.union(&curr.clone()).cloned().collect()
                });

                debug!("Inserting loc and neighbors into all matching areas");
                new_area.insert(*loc);
                for neighbor in &neighbors {
                    new_area.insert(*neighbor);
                }
                for matching_area in matching_areas {
                    if let Some(idx) = areas.iter().position(|area| area == &matching_area) {
                        areas.remove(idx);
                    } else {
                        debug!("Tried to remove matching_area, but couldn't find it in areas")
                    }
                }
                areas.push(new_area);
            }
        }

        result.insert(char, areas);
    }

    result
}

const NEIGHBOR_DIRECTIONS: [IVec2; 4] = [IVec2::X, IVec2::Y, IVec2::NEG_X, IVec2::NEG_Y];

fn parse(input: &str) -> HashMap<IVec2, char> {
    input
        .lines()
        .enumerate()
        .flat_map(|(y, row)| {
            row.char_indices()
                .map(move |(x, char)| (IVec2::new(x as i32, y as i32), char))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::IVec2;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn test_process_example_1() -> miette::Result<()> {
        let input = r#"
AAAA
BBCD
BBCC
EEEC
        "#
        .trim();
        assert_eq!(process(input)?, "80");
        Ok(())
    }

    #[test]
    fn test_process_example_2() -> miette::Result<()> {
        let input = r#"
EEEEE
EXXXX
EEEEE
EXXXX
EEEEE
        "#
        .trim();
        assert_eq!(process(input)?, "236");
        Ok(())
    }

    #[test]
    fn test_process_example_3() -> miette::Result<()> {
        let input = r#"
AAAAAA
AAABBA
AAABBA
ABBAAA
ABBAAA
AAAAAA
        "#
        .trim();
        assert_eq!(process(input)?, "368");
        Ok(())
    }

    #[test]
    fn test_process_example_4() -> miette::Result<()> {
        let input = r#"
RRRRIICCFF
RRRRIICCCF
VVRRRCCFFF
VVRCCCJFFF
VVVVCJJCFE
VVIVCCJJEE
VVIIICJJEE
MIIIIIJJEE
MIIISIJEEE
MMMISSJEEE
        "#
        .trim();
        assert_eq!(process(input)?, "1206");
        Ok(())
    }

    #[test]
    fn test_my_example_1() -> miette::Result<()> {
        let input = r#"
AAAA
        "#
        .trim();

        let parsed_tiles: HashMap<IVec2, char> = parse(input);
        dbg!(&parsed_tiles);
        let all_areas: HashMap<char, Vec<HashSet<IVec2>>> = find_areas(parsed_tiles.clone());
        assert_eq!(1, all_areas[&'A'].len());
        assert_eq!(4, all_areas[&'A'][0].len());

        // we should get the exact same map from the areas
        let tiles_from_areas: HashMap<IVec2, char> = all_areas
            .clone()
            .into_iter()
            .flat_map(|(char, areas_for_char)| {
                areas_for_char
                    .into_iter()
                    .flat_map(move |area| area.into_iter().map(move |pos| (pos, char)))
            })
            .collect();

        let scores = score_areas(&all_areas, parsed_tiles.clone());
        let scored_with_edges: Vec<ScoredArea> = add_edge_score_to_areas(&scores);

        dbg!(scores);

        assert_eq!(parsed_tiles, tiles_from_areas);

        assert_eq!(process(input)?, "16");

        Ok(())
    }

    #[test]
    fn test_my_example_2() -> miette::Result<()> {
        let input = r#"
AAABA
CCAAA
        "#
        .trim();

        let parsed_tiles: HashMap<IVec2, char> = parse(input);
        dbg!(&parsed_tiles);
        let all_areas: HashMap<char, Vec<HashSet<IVec2>>> = find_areas(parsed_tiles.clone());
        assert_eq!(1, all_areas[&'A'].len());
        assert_eq!(7, all_areas[&'A'][0].len());

        // we should get the exact same map from the areas
        let tiles_from_areas: HashMap<IVec2, char> = all_areas
            .clone()
            .into_iter()
            .flat_map(|(char, areas_for_char)| {
                areas_for_char
                    .into_iter()
                    .flat_map(move |area| area.into_iter().map(move |pos| (pos, char)))
            })
            .collect();

        let scores = score_areas(&all_areas, parsed_tiles.clone());
        let scored_with_edges: Vec<ScoredArea> = add_edge_score_to_areas(&scores);

        dbg!(scores);

        assert_eq!(parsed_tiles, tiles_from_areas);

        Ok(())
    }
}
