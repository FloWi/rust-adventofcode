use std::collections::{HashMap, HashSet};
use std::ops::{Add, Neg};
use glam::IVec2;
use itertools::Itertools;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (occupancy_map, location, direction) = parse_map(input);

    dbg!(&occupancy_map);
    dbg!(&location);
    dbg!(&direction);

    let visited = walk_off_the_earth(&occupancy_map, &location, &direction);

    let result = visited.len();

    Ok(result.to_string())
}

fn walk_off_the_earth(occupancy_map: &Vec<Vec<bool>>, location: &IVec2, direction: &IVec2) -> HashSet<IVec2> {

    let height = occupancy_map.len() as i32;
    let width = occupancy_map[0].len() as i32;

    let mut location = *location;
    let mut direction = *direction;
    let mut visited: HashSet<IVec2> = HashSet::from([location]);
    loop {
        //FIXME: might need to rotate multiple times if you hit a dead-end
        println!("Exploring {:?} in direction {:?}", location, direction);
        let lookup_location = location.add(direction);
        let is_occupied = occupancy_map.get(lookup_location.y as usize).and_then(|row| row.get(lookup_location.x as usize)).unwrap_or(&false);
        if *is_occupied {
            direction = IVec2::new(-direction.y, direction.x); // rotate 90° CW if you hit an obstacle. Glam's positive y-axis points up, so we can't use their internal rotation stuff
            println!("Hit obstacle - new direction is {:?}", direction);
        }
        location = location.add(direction);
        if location.y < 0 || location.y >= height || location.x < 0 || location.x >= width {
            println!("Fell off the earth at {:}", location);
            return visited
        }
        visited.insert(location);
    }
}

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
