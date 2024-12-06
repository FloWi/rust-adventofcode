use glam::IVec2;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::ops::{Add, Mul, Neg};

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (occupancy_map, location, direction) = parse_map(input);

    //dbg!(&occupancy_map);
    //dbg!(&location);
    //dbg!(&direction);

    let (obstacles, path) = walk_off_the_earth(&occupancy_map, &location, &direction);

    //dbg!(&obstacles);

    let result = obstacles.len();

    let expected = HashSet::from([
        IVec2::new(3, 6),
        IVec2::new(6, 7),
        IVec2::new(7, 7),
        IVec2::new(1, 8),
        IVec2::new(3, 8),
        IVec2::new(7, 9),
    ]);

    let missing = expected.difference(&obstacles);
    let not_expected = obstacles.difference(&expected);
    let correct = expected.intersection(&obstacles);

    //println!("Path: {:?}", path);
    //println!("expected: {:?}", &expected);
    //println!("missing: {:?}", &missing);
    //println!("not_expected: {:?}", &not_expected);
    //println!("correct: {:?}", &correct);

    //assert_eq!(&expected, &obstacles);

    Ok(result.to_string())
}

fn walk_off_the_earth(
    occupancy_map: &Vec<Vec<bool>>,
    location: &IVec2,
    direction: &IVec2,
) -> (HashSet<IVec2>, Vec<IVec2>) {
    let height = occupancy_map.len() as i32;
    let width = occupancy_map[0].len() as i32;
    let max_dimension = height.max(width);

    let start = location;

    let mut location = *location;
    let mut direction = *direction;
    let mut visited: HashSet<(IVec2, IVec2)> = HashSet::from([(location, direction)]);
    let mut obstacles: HashSet<IVec2> = HashSet::new();
    let mut path: Vec<IVec2> = Vec::new();

    loop {
        //FIXME: might need to rotate multiple times if you hit a dead-end
        //println!("Exploring {:?} in direction {:?}", location, direction);
        let lookup_location = location.add(direction);
        let is_occupied = occupancy_map
            .get(lookup_location.y as usize)
            .and_then(|row| row.get(lookup_location.x as usize))
            .unwrap_or(&false);
        if *is_occupied {
            direction = IVec2::new(-direction.y, direction.x); // rotate 90° CW if you hit an obstacle. Glam's positive y-axis points up, so we can't use their internal rotation stuff
                                                               //println!("Hit obstacle - new direction is {:?}", direction);
        }
        location = location.add(direction);
        if location.y < 0 || location.y >= height || location.x < 0 || location.x >= width {
            //println!("Fell off the earth at {:}", location);
            return (obstacles, path);
        }

        visited.insert((location, direction));
        path.push(location);
        // check if a tile if right-hand direction has already been visited. If so, an obstacle placed ahead of us would send us in an infinite loop
        let rhs_direction = IVec2::new(-direction.y, direction.x);
        let obstacle_location = location.add(direction);
        let hit_previous_path = (1..max_dimension).into_iter().any(|offset| {
            let offset_location = rhs_direction.mul(offset);
            let check_location = location + offset_location;
            let has_been_visited = visited.contains(&(check_location, rhs_direction));
            has_been_visited
        });
        if hit_previous_path && obstacle_location != *start {
            obstacles.insert(obstacle_location);
        }
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
        assert_eq!("6", process(input)?);
        Ok(())
    }
}
