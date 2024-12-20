use crate::{find_path, parse};

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let racetrack = parse(input);

    let (_path, cost) =
        find_path(&racetrack.walls, &racetrack.start, &racetrack.end).expect("path to be found");

    println!("Found path of length {}", cost);

    let result = 42;

    Ok(result.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
###############
#...#...#.....#
#.#.#.#.#.###.#
#S#...#.#.#...#
#######.#.#.###
#######.#.#...#
#######.#.###.#
###..E#...#...#
###.#######.###
#...###...#...#
#.#####.#.###.#
#.#...#.#.#...#
#.#.#.#.#.#.###
#...#...#...###
###############
        "#
        .trim();

        // There are 14 cheats that save 2 picoseconds.
        // There are 14 cheats that save 4 picoseconds.
        // There are 2 cheats that save 6 picoseconds.
        // There are 4 cheats that save 8 picoseconds.
        // There are 2 cheats that save 10 picoseconds.
        // There are 3 cheats that save 12 picoseconds.
        // There is one cheat that saves 20 picoseconds.
        // There is one cheat that saves 36 picoseconds.
        // There is one cheat that saves 38 picoseconds.
        // There is one cheat that saves 40 picoseconds.
        // There is one cheat that saves 64 picoseconds.

        let expected_number_of_cheats: u32 = vec![14, 14, 2, 4, 2, 3, 1, 1, 1, 1, 1].iter().sum();
        assert_eq!(expected_number_of_cheats.to_string(), process(input)?);
        Ok(())
    }

    #[test]
    fn pathfinding() {
        let input = r#"
###############
#...#...#.....#
#.#.#.#.#.###.#
#S#...#.#.#...#
#######.#.#.###
#######.#.#...#
#######.#.###.#
###..E#...#...#
###.#######.###
#...###...#...#
#.#####.#.###.#
#.#...#.#.#...#
#.#.#.#.#.#.###
#...#...#...###
###############
        "#
        .trim();

        let racetrack = parse(input);

        let (_path, cost) = find_path(&racetrack.walls, &racetrack.start, &racetrack.end)
            .expect("path to be found");

        assert_eq!(cost, 84);
    }
}
