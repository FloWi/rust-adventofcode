use glam::{IVec2, UVec2};
use itertools::Itertools;
use pathfinding::matrix::Matrix;
use pathfinding::prelude::dijkstra;
use std::collections::HashMap;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let maze = parse(input);

    let matrix = Matrix::from_fn(maze.height as usize, maze.width as usize, |(y, x)| {
        maze.map.get(&UVec2::new(x as u32, y as u32)).unwrap()
    });

    let successors = |pos: &(usize, usize)| {
        matrix
            .neighbours(*pos, false)
            .filter(|next_pos| match matrix.get(*next_pos).unwrap() {
                Tile::Wall => false,
                Tile::Empty => true,
            })
            .map(|next_pos| (next_pos, 1)) // Cost of 1 for each move
            .collect::<Vec<_>>()
    };

    if let Some(pathfinding_result) = dijkstra(
        &(maze.start_pos.y as usize, maze.start_pos.x as usize),
        successors,
        |(y, x)| *y == maze.end_pos.y as usize && *x == maze.start_pos.x as usize,
    ) {
        let score: u32 = compute_score(IVec2::X, pathfinding_result);
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
        .chain(path.into_iter().tuple_windows().map(|(from, to)| {
            let dir = to - from;
            dir
        }))
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
    map: HashMap<UVec2, Tile>,
    start_pos: UVec2,
    end_pos: UVec2,
    width: u32,
    height: u32,
}

fn parse(input: &str) -> Maze {
    let mut map = HashMap::new();
    let mut start_pos = None;
    let mut end_pos = None;

    for (y, row) in input.lines().enumerate() {
        for (x, char) in row.char_indices() {
            let pos = UVec2::new(x as u32, y as u32);
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
        width: &map.keys().map(|loc| loc.x).max().unwrap() + 1,
        height: &map.keys().map(|loc| loc.y).max().unwrap() + 1,
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
        assert_eq!("7036", process(input)?);
        Ok(())
    }
}
