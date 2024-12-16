use glam::IVec2;
use itertools::Itertools;
use pathfinding::prelude::astar_bag_collect;
use std::collections::HashMap;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let maze = parse(input);

    let successors = |(current_pos, direction): &(IVec2, IVec2)| {
        let new_forward_location = current_pos + direction;

        let is_horizontal = direction.y == 0;
        let other_directions = if is_horizontal {
            vec![IVec2::Y, IVec2::NEG_Y]
        } else {
            vec![IVec2::X, IVec2::NEG_X]
        };

        let forward_neighbor_vec: Vec<((IVec2, IVec2), i32)> =
            match maze.map.get(&new_forward_location) {
                None => {
                    panic!("shouldn't happen")
                }
                Some(tile) => match tile {
                    Tile::Wall => {
                        vec![]
                    }
                    Tile::Empty => {
                        vec![((new_forward_location, *direction), 1)]
                    }
                },
            };

        let other_directions_neighbors: Vec<((IVec2, IVec2), i32)> = other_directions
            .into_iter()
            .map(|new_dir| ((*current_pos, new_dir), 1000))
            .collect_vec();

        forward_neighbor_vec
            .into_iter()
            .chain(other_directions_neighbors)
    };

    // set heuristic to 0 (will make it behave like dijkstra). Should be
    if let Some((all_best_paths, _score)) = astar_bag_collect(
        &(maze.start_pos, IVec2::X),
        successors,
        |(pos, dir)| (maze.end_pos - pos).abs().element_sum(), //manhattan distance as heuristic
        |(pos, _dir)| pos == &maze.end_pos,
    ) {
        let score = all_best_paths
            .into_iter()
            .flat_map(|path| path.into_iter().map(|(pos, _dir)| pos))
            .unique()
            .count();
        Ok(score.to_string())
    } else {
        panic!("no path found")
    }
}

fn compute_score(starting_direction: IVec2, (path, _): (Vec<(usize, usize)>, i32)) -> u32 {
    let path = path
        .into_iter()
        .map(|(y, x)| IVec2::new(x as i32, y as i32))
        .collect_vec();
    dbg!(&path);

    let directions = vec![starting_direction]
        .into_iter()
        .chain(path.into_iter().tuple_windows().map(|(from, to)| to - from))
        .collect_vec();

    dbg!(&directions);

    let score = directions
        .iter()
        .tuple_windows()
        .map(
            |(current, next)| {
                if current == next {
                    1u32
                } else {
                    1000u32
                }
            },
        )
        .sum();

    score
}

#[derive(Debug, Clone)]
enum Tile {
    Wall,
    Empty,
}

#[derive(Debug)]
struct Maze {
    map: HashMap<IVec2, Tile>,
    start_pos: IVec2,
    end_pos: IVec2,
    width: u32,
    height: u32,
}

fn parse(input: &str) -> Maze {
    let mut map = HashMap::new();
    let mut start_pos = None;
    let mut end_pos = None;

    for (y, row) in input.lines().enumerate() {
        for (x, char) in row.char_indices() {
            let pos = IVec2::new(x as i32, y as i32);
            match char {
                '#' => {
                    map.insert(pos, Tile::Wall);
                }
                '.' | 'S' | 'E' => {
                    map.insert(pos, Tile::Empty);
                    (char == 'S').then(|| start_pos = Some(pos));
                    (char == 'E').then(|| end_pos = Some(pos));
                }
                unknown => panic!("Cannot parse unknown char {unknown}"),
            }
        }
    }

    Maze {
        map: map.clone(),
        start_pos: start_pos.unwrap(),
        end_pos: end_pos.unwrap(),
        width: map.keys().map(|loc| loc.x).max().unwrap() as u32 + 1,
        height: map.keys().map(|loc| loc.y).max().unwrap() as u32 + 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
###############
#.......#....E#
#.#.###.#.###.#
#.....#.#...#.#
#.###.#####.#.#
#.#.#.......#.#
#.#.#####.###.#
#...........#.#
###.#.#####.#.#
#...#.....#.#.#
#.#.#.###.#.#.#
#.....#...#.#.#
#.###.#.#.#.#.#
#S..#.....#...#
###############
        "#
        .trim();
        assert_eq!("45", process(input)?);
        Ok(())
    }
}
