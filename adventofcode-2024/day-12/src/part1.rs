use glam::IVec2;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let parsed_tiles: HashMap<IVec2, char> = parse(input);
    dbg!(&parsed_tiles);
    let all_areas: HashMap<char, Vec<HashSet<IVec2>>> = find_areas(parsed_tiles.clone());

    let scores = score_areas(&all_areas, parsed_tiles.clone());

    dbg!(&scores);

    let result: usize = scores.iter().map(|scored| scored.score).sum();

    Ok(result.to_string())
}

#[derive(Debug)]
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
                .iter().filter(|&area| {
                    area.contains(loc) || area.iter().any(|node| neighbors.contains(node))
                }).cloned()
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
    fn test_process() -> miette::Result<()> {
        let input = r#"
AAAA
BBCD
BBCC
EEEC
        "#
        .trim();
        assert_eq!("140", process(input)?);
        Ok(())
    }

    #[test]
    fn test_example_1() -> miette::Result<()> {
        let input = r#"
AAAA
BBCD
BBCC
EEEC
        "#
        .trim();

        let parsed_tiles: HashMap<IVec2, char> = parse(input);
        dbg!(&parsed_tiles);
        let all_areas: HashMap<char, Vec<HashSet<IVec2>>> = find_areas(parsed_tiles.clone());
        assert_eq!(1, all_areas[&'A'].len());
        assert_eq!(1, all_areas[&'B'].len());
        assert_eq!(1, all_areas[&'C'].len());
        assert_eq!(1, all_areas[&'D'].len());
        assert_eq!(1, all_areas[&'E'].len());

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

        dbg!(scores);

        assert_eq!(parsed_tiles, tiles_from_areas);

        Ok(())
    }
}
