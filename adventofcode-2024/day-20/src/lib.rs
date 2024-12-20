use glam::IVec2;
use itertools::Itertools;
use pathfinding::prelude::astar;
use std::collections::HashSet;
use std::ops::RangeInclusive;

pub mod part1;
pub mod part2;

fn find_path(walls: &HashSet<IVec2>, start: &IVec2, goal: &IVec2) -> Option<(Vec<IVec2>, u32)> {
    let grid_limit_xs: &RangeInclusive<i32> = &(0..=walls.iter().map(|pos| pos.x).max()?);
    let grid_limit_ys: &RangeInclusive<i32> = &(0..=walls.iter().map(|pos| pos.y).max()?);

    astar(
        start,
        |&loc| {
            NEIGHBORS.iter().filter_map({
                move |offset| {
                    let new_loc = loc + offset;
                    let is_in_grid =
                        grid_limit_xs.contains(&new_loc.x) && grid_limit_ys.contains(&new_loc.y);
                    let is_blocked = walls.contains(&new_loc);
                    (is_in_grid && !is_blocked).then_some((new_loc, 1))
                }
            })
        },
        |loc| (loc.distance_squared(*goal) as f32).sqrt().round() as u32,
        |loc| loc == goal,
    )
}

const NEIGHBORS: [IVec2; 4] = [IVec2::NEG_Y, IVec2::X, IVec2::Y, IVec2::NEG_X];

#[derive(Debug)]
pub struct Racetrack {
    start: IVec2,
    end: IVec2,
    walls: HashSet<IVec2>,
}

fn parse(input: &str) -> Racetrack {
    #[derive(Debug)]
    enum ParsedTile {
        Empty,
        Wall(IVec2),
        Start(IVec2),
        End(IVec2),
    }

    let parsed_tiles = input
        .lines()
        .enumerate()
        .flat_map(|(y, row)| {
            row.char_indices().map(move |(x, char)| {
                let loc = IVec2::new(x as i32, y as i32);
                match char {
                    'S' => ParsedTile::Start(loc),
                    'E' => ParsedTile::End(loc),
                    '.' => ParsedTile::Empty,
                    '#' => ParsedTile::Wall(loc),
                    tile => panic!("can't parse tile '{tile}'"),
                }
            })
        })
        .collect_vec();

    let (start, end, walls) = parsed_tiles.iter().fold(
        (None, None, HashSet::new()),
        |(start, end, mut walls), tile| match tile {
            ParsedTile::Empty => (start, end, walls),
            ParsedTile::Wall(pos) => {
                walls.insert(*pos);
                (start, end, walls)
            }
            ParsedTile::Start(pos) => (Some(pos), end, walls),
            ParsedTile::End(pos) => (start, Some(pos), walls),
        },
    );

    Racetrack {
        start: *start.unwrap(),
        end: *end.unwrap(),
        walls,
    }
}
