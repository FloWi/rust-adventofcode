use glam::IVec2;
use nom::character::complete;
use nom::character::complete::{char, line_ending};
use nom::combinator::map;
use nom::multi::separated_list1;
use nom::sequence::separated_pair;
use nom::IResult;
use pathfinding::prelude::astar;
use std::collections::HashSet;
use std::ops::RangeInclusive;

pub mod part1;
pub mod part2;

fn find_path(
    byte_locations: &Vec<IVec2>,
    goal: &IVec2,
    num_bytes: usize,
    grid_limit: &RangeInclusive<i32>,
) -> Option<(Vec<IVec2>, u32)> {
    let occupied: &HashSet<IVec2> = &byte_locations.clone().into_iter().take(num_bytes).collect();

    astar(
        &IVec2::ZERO,
        |&loc| {
            NEIGHBORS.iter().filter_map({
                move |offset| {
                    let new_loc = loc + offset;
                    let is_in_grid =
                        grid_limit.contains(&new_loc.x) && grid_limit.contains(&new_loc.y);
                    let is_blocked = occupied.contains(&new_loc);
                    (is_in_grid && !is_blocked).then_some((new_loc, 1))
                }
            })
        },
        |loc| (loc.distance_squared(*goal) as f32).sqrt().round() as u32,
        |loc| loc == goal,
    )
}

const NEIGHBORS: [IVec2; 4] = [IVec2::NEG_Y, IVec2::X, IVec2::Y, IVec2::NEG_X];

fn parse(input: &str) -> IResult<&str, Vec<IVec2>> {
    separated_list1(
        line_ending,
        map(
            separated_pair(complete::i32, char(','), complete::i32),
            |(x, y)| IVec2::new(x, y),
        ),
    )(input)
}
