use glam::IVec2;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let parsed_tiles: HashMap<IVec2, char> = parse(input);
    dbg!(&parsed_tiles);
    let all_areas: HashMap<char, Vec<HashSet<IVec2>>> = find_areas(parsed_tiles.clone());

    let scores = score_areas(&all_areas, parsed_tiles.clone());
    let scored_with_edges: Vec<(ScoredArea, usize)> = add_edge_score_to_areas(&scores);

    dbg!(&scored_with_edges);

    let result: usize = scored_with_edges
        .iter()
        .map(|(scored_area, num_sides)| scored_area.size * num_sides)
        .sum();

    Ok(result.to_string())
}

#[derive(Debug, Hash, Eq, PartialEq)]
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

fn add_edge_score_to_areas(scored_areas: &Vec<ScoredArea>) -> Vec<(ScoredArea, usize)> {
    scored_areas
        .clone()
        .into_iter()
        .map(|scored_area| {
            let edges = scored_area
                .area
                .iter()
                .flat_map(|loc| {
                    EDGE_OFFSETS.iter().map(move |(dir, offset)| {
                        // these edges are identical
                        // 0,0 SOUTH
                        // 0,1 NORTH
                        // so we normalize these edges
                        // SOUTH edge of x == NORTH edge of SOUTH_NEIGHBOR
                        // WEST edge of x == EAST edge of WEST_NEIGHBOR
                        let (normalized_direction, location) = match dir {
                            EdgeDir::North => (dir, *loc),
                            EdgeDir::East => (dir, *loc),
                            EdgeDir::South => (&EdgeDir::North, loc + offset),
                            EdgeDir::West => (&EdgeDir::East, loc + offset),
                        };
                        (location, normalized_direction)
                    })
                })
                .collect_vec();

            let outer_edges = edges
                .into_iter()
                .counts()
                .into_iter()
                .filter(|(edge, count)| *count < 2)
                .collect_vec();
            let edges_string = outer_edges
                .clone()
                .into_iter()
                .map(|((loc, edge_dir), count)| {
                    let direction_label = match edge_dir {
                        EdgeDir::North => "N".to_string(),
                        EdgeDir::East => "E".to_string(),
                        EdgeDir::South => "S".to_string(),
                        EdgeDir::West => "W".to_string(),
                    };
                    format!("{loc}{direction_label}")
                })
                .join(", ");

            //let edges

            println!("outer edges for '{}': {edges_string}", scored_area.label);

            let edge_groups: Vec<Vec<IVec2>> =
                combine_edges_into_adjacent_edge_groups(&outer_edges, &scored_area);

            (scored_area, edge_groups.len())
        })
        .collect_vec()
}

fn combine_edges_into_adjacent_edge_groups(
    outer_edges: &Vec<((IVec2, &EdgeDir), usize)>,
    scored_area: &ScoredArea,
) -> Vec<Vec<IVec2>> {
    println!(
        "\n\ncombine_edges_into_adjacent_edge_groups for '{}'",
        scored_area.label
    );

    let mut result = Vec::new();

    let horizontal_edge_groups_by_y = outer_edges
        .iter()
        .filter(|((loc, edge_dir), count)| **edge_dir == EdgeDir::North)
        .map(|((loc, edge_dir), count)| (loc, edge_dir))
        .into_group_map_by(|(loc, _)| loc.y)
        .into_iter()
        .collect_vec();

    let vertical_edge_groups_by_x = outer_edges
        .iter()
        .filter(|((loc, edge_dir), count)| **edge_dir == EdgeDir::East)
        .map(|((loc, edge_dir), count)| (loc, edge_dir))
        .into_group_map_by(|(loc, _)| loc.x)
        .into_iter()
        .collect_vec();

    for (x, candidates) in vertical_edge_groups_by_x.iter() {
        let consecutive_edges: Vec<Vec<IVec2>> = create_adjacent_groups_of_sorted_candidates(
            &candidates
                .iter()
                .map(|(loc, _)| **loc)
                .sorted_by_key(|loc| loc.y)
                .collect_vec(),
        );

        println!(
            "found {} vertical edges for x = {x}\nedges: {}",
            consecutive_edges.len(),
            consecutive_edges
                .iter()
                .cloned()
                .map(|edge_group| edge_group.iter().join(", "))
                .join("\n")
        );
        for edge_group in consecutive_edges {
            result.push(edge_group)
        }
    }

    for (y, candidates) in horizontal_edge_groups_by_y.iter() {
        let consecutive_edges: Vec<Vec<IVec2>> = create_adjacent_groups_of_sorted_candidates(
            &candidates
                .iter()
                .map(|(loc, _)| **loc)
                .sorted_by_key(|loc| loc.x)
                .collect_vec(),
        );

        println!(
            "found {} horizontal edges for y = {y}\nedges: {}\n\n",
            consecutive_edges.len(),
            consecutive_edges
                .iter()
                .cloned()
                .map(|edge_group| edge_group.iter().join(", "))
                .join("\n")
        );
        for edge_group in consecutive_edges {
            result.push(edge_group)
        }
    }

    result
}

fn create_adjacent_groups_of_sorted_candidates(nodes: &[IVec2]) -> Vec<Vec<IVec2>> {
    let mut result = Vec::new();

    let mut open_list = nodes.iter().cloned().collect_vec();

    loop {
        dbg!(&open_list);
        if let Some(start) = open_list.first() {
            let neighbors_with_distance_to_start: Vec<(IVec2, i32)> = open_list
                .iter()
                .map(|n| {
                    let diff = n - start;
                    let distance = diff.x.abs() + diff.y.abs();
                    (*n, distance)
                })
                .enumerate()
                //.inspect(|x| println!("(idx, (n, distance)): {x:?}"))
                .take_while(|(idx, (n, distance))| *idx as i32 == *distance)
                .map(|(idx, (n, distance))| (n, distance))
                .collect_vec();

            //dbg!(&neighbors_with_distance_to_start);

            let group = neighbors_with_distance_to_start
                .into_iter()
                .map(|tup| tup.0)
                .collect_vec();

            result.push(group.clone());
            open_list = open_list.into_iter().skip(group.len()).collect_vec();
        } else {
            return result;
        }
    }
}

#[derive(Debug, Clone)]
struct ScoredArea {
    area: HashSet<IVec2>,
    label: char,
    size: usize,
    perimeter: usize,
    score: usize,
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
        println!(
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

            println!("Evaluating loc {loc}. Found {} neighbor(s). Loc and or neighbors are contained in {} area(s)", neighbors.len(), matching_areas.len());

            if matching_areas.is_empty() {
                let mut new_area = HashSet::new();
                new_area.insert(*loc);
                for neighbor in neighbors.iter() {
                    new_area.insert(*neighbor);
                }
                println!("No matching area found. Creating new one with location and all relevant neighbors: {new_area:?}");
                areas.push(new_area);
            } else {
                // we can now merge the areas together, since we found connecting tile(s) between all of them.
                let mut new_area = matching_areas.iter().fold(HashSet::new(), |acc, curr| {
                    acc.union(&curr.clone()).cloned().collect()
                });

                println!("Inserting loc and neighbors into all matching areas");
                new_area.insert(*loc);
                for neighbor in &neighbors {
                    new_area.insert(*neighbor);
                }
                for matching_area in matching_areas {
                    if let Some(idx) = areas.iter().position(|area| area == &matching_area) {
                        areas.remove(idx);
                    } else {
                        println!("Tried to remove matching_area, but couldn't find it in areas")
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
        let scored_with_edges: Vec<(ScoredArea, usize)> = add_edge_score_to_areas(&scores);

        dbg!(scores);

        assert_eq!(parsed_tiles, tiles_from_areas);

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
        let scored_with_edges: Vec<(ScoredArea, usize)> = add_edge_score_to_areas(&scores);

        dbg!(scores);

        assert_eq!(parsed_tiles, tiles_from_areas);

        Ok(())
    }
}
