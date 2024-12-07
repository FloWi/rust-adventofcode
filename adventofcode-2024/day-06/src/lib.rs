use glam::IVec2;
use std::collections::{HashMap, HashSet};
use std::ops::Add;

pub mod part1;
pub mod part2;

fn parse_map(input: &str) -> (Vec<Vec<bool>>, IVec2, IVec2) {
    let mut occupancy_map = Vec::new();
    let mut location: Option<IVec2> = None;
    let mut direction: Option<IVec2> = None;

    let directions = HashMap::from([
        ('>', IVec2::new(1, 0)),
        ('v', IVec2::new(0, 1)),
        ('<', IVec2::new(-1, 0)),
        ('^', IVec2::new(0, -1)),
    ]);

    for (y, line_str) in input.lines().enumerate() {
        if &occupancy_map.len() <= &y {
            &occupancy_map.push(Vec::new());
        }
        let mut line = &mut occupancy_map[y];
        for (x, map_char) in line_str.char_indices() {
            if map_char == '#' {
                line.push(true)
            } else {
                if let Some(dir) = directions.get(&map_char) {
                    location = Some(IVec2::new(x as i32, y as i32));
                    direction = Some(*dir);
                }
                line.push(false)
            }
        }
    }
    (occupancy_map, location.unwrap(), direction.unwrap())
}

fn walk_off_the_earth<F>(
    occupancy_map: &Vec<Vec<bool>>,
    location: &IVec2,
    direction: &IVec2,
    extra_obstacle: Option<IVec2>,
    in_bounds: F,
) -> (HashSet<IVec2>, Vec<IVec2>)
where
    F: Fn(IVec2) -> bool,
{
    let mut location = *location;
    let mut direction = *direction;
    let mut visited: HashSet<IVec2> = HashSet::from([location]);
    let mut path: Vec<IVec2> = vec![location];
    loop {
        let (new_location, new_direction) = perform_step(occupancy_map, &location, &direction, extra_obstacle);
        location = new_location;
        direction = new_direction;
        if !in_bounds(location) {
            return (visited, path);
        }
        if !visited.contains(&location) {
            path.push(location);
        }
        visited.insert(location);

    }
}

fn perform_step(occupancy_map: &Vec<Vec<bool>>, location: &IVec2, direction: &IVec2, extra_obstacle: Option<IVec2>) -> (IVec2, IVec2) {
    let lookup_location = location.add(direction);
    let maybe_hit_extra_obstacle = extra_obstacle.map(|ex| ex == lookup_location);
    let is_occupied = occupancy_map
        .get(lookup_location.y as usize)
        .and_then(|row| row.get(lookup_location.x as usize))
        .or(maybe_hit_extra_obstacle.as_ref())
        .unwrap_or(&false);
    let new_direction = if *is_occupied {
        IVec2::new(-direction.y, direction.x) // rotate 90° CW if you hit an obstacle. Glam's positive y-axis points up, so we can't use their internal rotation stuff
    } else {
        *direction
    };

    let new_location = if *is_occupied {
        *location
    } else {
        location.add(direction)
    };

    (new_location, new_direction)
}


