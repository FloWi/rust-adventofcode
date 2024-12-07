use crate::{parse_map, walk_off_the_earth};
use glam::IVec2;
use itertools::Itertools;
use std::ops::{Add, Neg};

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (occupancy_map, location, direction) = parse_map(input);

    let height = occupancy_map.len();
    let width = occupancy_map[0].len();

    let map_dimensions = IVec2::new(width as i32, height as i32);

    let is_in_bounds = |loc: IVec2| {
        loc.x >= 0 && loc.y >= 0 && loc.x < map_dimensions.x && loc.y < map_dimensions.y
    };

    let (visited, _) =
        walk_off_the_earth(&occupancy_map, &location, &direction, None, is_in_bounds);

    let result = visited.len();

    Ok(result.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
....#.....
.........#
..........
..#.......
.......#..
..........
.#..^.....
........#.
#.........
......#...
        "#
        .trim();
        assert_eq!("41", process(input)?);
        Ok(())
    }
}
