use glam::IVec2;
use itertools::Itertools;
use std::collections::HashSet;
use std::ops::{Add, Mul, Neg};

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (occupancy_map, starting_location, direction) = parse_map(input);

    //dbg!(&occupancy_map);
    //dbg!(&location);
    //dbg!(&direction);


    let height = occupancy_map.len();
    let width = occupancy_map[0].len();

    let map_dimensions = IVec2::new(width as i32, height as i32);

    let is_in_bounds = |loc: IVec2| loc.x >= 0 && loc.y >= 0 && loc.x < map_dimensions.x && loc.y < map_dimensions.y;

    let (visited, original_path) = walk_off_the_earth(&occupancy_map, &starting_location, &direction, None, is_in_bounds);

    let correct_obstacles = original_path.into_iter()
        .filter(|extra_obstacle| extra_obstacle != &starting_location)
        //.inspect(|potential_extra_obstacle| { dbg!(potential_extra_obstacle); })
        .filter_map(|extra_obstacle| {
            let is_loop = find_loop(&occupancy_map, &starting_location, &direction, extra_obstacle, is_in_bounds);
            Some(extra_obstacle).filter(|_| is_loop)
        })
        //.inspect(|extra_obstacle| { dbg!(extra_obstacle); })
        .collect_vec();

    let result = correct_obstacles.len();
    Ok(result.to_string())
}


fn find_loop<F>(occupancy_map: &Vec<Vec<bool>>,
                starting_location: &IVec2,
                starting_direction: &IVec2,
                extra_obstacle: IVec2,
                in_bounds: F,
) -> bool
where
    F: Fn(IVec2) -> bool,
{
    let mut location = *starting_location;
    let mut direction = *starting_direction;
    let mut visited: HashSet<(IVec2, IVec2)> = HashSet::from([(location, direction)]);
    loop {
        //FIXME: might need to rotate multiple times if you hit a dead-end
        let (new_location, new_direction) = perform_step(occupancy_map, &location, &direction, Some(extra_obstacle));
        location = new_location;
        direction = new_direction;
        let is_in_bounds = in_bounds(location);
        let has_been_visited = visited.contains(&(location, direction));

        if !is_in_bounds {
            return false;
        }
        if has_been_visited {
            return true;
        } else {
            visited.insert((location, direction));
        }
    }
}


use crate::{parse_map, perform_step, walk_off_the_earth};
use std::fmt::Write;

fn generate_svg(
    path: &[IVec2],
    original_obstacles: &[IVec2],
    new_obstacles: &[IVec2],
    cell_size: i32,
) -> String {
    // Find bounds
    let all_points: Vec<_> = path.iter()
        .chain(new_obstacles.iter())
        .collect();

    let min_x = all_points.iter().map(|p| p.x).min().unwrap_or(0) - 1;
    let max_x = all_points.iter().map(|p| p.x).max().unwrap_or(0) + 1;
    let min_y = all_points.iter().map(|p| p.y).min().unwrap_or(0) - 1;
    let max_y = all_points.iter().map(|p| p.y).max().unwrap_or(0) + 1;

    let width = (max_x - min_x + 1) * cell_size;
    let height = (max_y - min_y + 1) * cell_size;

    let mut svg = String::new();

    // SVG header
    writeln!(&mut svg, "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{} {} {} {}\">",
             min_x * cell_size, min_y * cell_size, width, height).unwrap();

    // Grid
    for x in min_x..=max_x {
        writeln!(&mut svg, "  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#ddd\" stroke-width=\"1\"/>",
                 x * cell_size, min_y * cell_size, x * cell_size, max_y * cell_size).unwrap();
    }
    for y in min_y..=max_y {
        writeln!(&mut svg, "  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#ddd\" stroke-width=\"1\"/>",
                 min_x * cell_size, y * cell_size, max_x * cell_size, y * cell_size).unwrap();
    }

    // Obstacles
    for pos in original_obstacles {
        writeln!(&mut svg, "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#666\"/>",
                 pos.x * cell_size, pos.y * cell_size, cell_size, cell_size).unwrap();
    }

    // Obstacles
    for pos in new_obstacles {
        writeln!(&mut svg, "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#FF0000\"/>",
                 pos.x * cell_size, pos.y * cell_size, cell_size, cell_size).unwrap();
    }

    // Function to get cell center
    let center = |pos: &IVec2| {
        (pos.x * cell_size + cell_size / 2, pos.y * cell_size + cell_size / 2)
    };

    // Main path lines
    for window in path.windows(2) {
        let (x1, y1) = center(&window[0]);
        let (x2, y2) = center(&window[1]);
        writeln!(&mut svg, "  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#2196F3\" stroke-width=\"2\" opacity=\"0.8\"/>",
                 x1, y1, x2, y2).unwrap();
    }


    // Main path points
    for (idx, pos) in path.iter().enumerate() {
        let x = pos.x * cell_size + (cell_size as f32 * 0.2) as i32;
        let y = pos.y * cell_size + (cell_size as f32 * 0.2) as i32;
        let size = (cell_size as f32 * 0.6) as i32;
        let color = if idx == 0 {
            "#00FF00"
        } else {
            "#2196F3"
        };
        writeln!(&mut svg, "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\" opacity=\"0.8\"/>",
                 x, y, size, size, color).unwrap();
    }

    // Close SVG
    writeln!(&mut svg, "</svg>").unwrap();

    svg
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
        assert_eq!("6", process(input)?);
        Ok(())
    }

    #[test]
    fn test_find_loop() -> miette::Result<()> {
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
        let (occupancy_map, starting_location, direction) = parse_map(input);

        //dbg!(&occupancy_map);
        //dbg!(&location);
        //dbg!(&direction);


        let height = occupancy_map.len();
        let width = occupancy_map[0].len();

        let map_dimensions = IVec2::new(width as i32, height as i32);

        let is_in_bounds = |loc: IVec2| loc.x >= 0 && loc.y >= 0 && loc.x < map_dimensions.x && loc.y < map_dimensions.y;

        let (visited, original_path) = walk_off_the_earth(&occupancy_map, &starting_location, &direction, None, is_in_bounds);
        let extra_obstacle = IVec2::new(3,6);
        let is_loop = find_loop(&occupancy_map, &starting_location, &direction, extra_obstacle, is_in_bounds);

        assert!(is_loop);

        Ok(())

    }
}
