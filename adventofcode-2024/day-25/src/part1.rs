use itertools::Itertools;
use tracing::info;

const WIDTH: usize = 5;
const HEIGHT: usize = 5;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let entities = parse(input);

    let (locks, keys): (Vec<_>, Vec<_>) =
        entities
            .iter()
            .partition(|entity| match entity.schema_type {
                SchemaType::Lock => true,
                SchemaType::Key => false,
            });

    info!("Found {} keys and {} locks", keys.len(), locks.len());

    let result = locks
        .iter()
        .cartesian_product(keys)
        .filter(|(lock, key)| is_match(lock, key))
        .count();
    Ok(result.to_string())
}

fn is_match(e1: &Entity, e2: &Entity) -> bool {
    let found_violation = (0..WIDTH).any(|idx| e1.heights[idx] + e2.heights[idx] > HEIGHT);
    !found_violation
}

fn parse(input: &str) -> Vec<Entity> {
    // The locks are schematics that have the top row filled (#) and the bottom row empty (.);
    // the keys have the top row empty and the bottom row filled.

    // So, you could say the first lock has pin heights 0,5,3,4,3:
    //   01234
    // 0 #####
    // 1 .####
    // 2 .####
    // 3 .####
    // 4 .#.#.
    // 5 .#...
    // 6 .....

    // Or, that the first key has heights 5,0,2,1,3:
    //   01234
    // 0 .....
    // 1 #....
    // 2 #....
    // 3 #...#
    // 4 #.#.#
    // 5 #.###
    // 6 #####

    input
        .split("\n\n")
        .map(|schema| {
            let mut lines_iter = schema.lines();
            let first_line = lines_iter.next().expect("Iterator must not be empty");

            let schema_type = if first_line == "#####" {
                SchemaType::Lock
            } else if first_line == "....." {
                SchemaType::Key
            } else {
                panic!("first line should be either Key(.....) or Lock(#####)")
            };

            let mut column_heights: [Option<usize>; WIDTH] = [None, None, None, None, None];

            // we consumed the first line already.
            // Now we're scanning down the columns and look for the first
            // . if we're scanning a lock
            // # if we're scanning a key
            lines_iter.enumerate().for_each(|(y, line)| {
                line.char_indices().for_each(|(x, char)| match schema_type {
                    SchemaType::Lock => {
                        if char == '.' && column_heights[x].is_none() {
                            column_heights[x] = Some(y)
                        }
                    }
                    SchemaType::Key => {
                        if char == '#' && column_heights[x].is_none() {
                            column_heights[x] = Some(HEIGHT - y)
                        }
                    }
                })
            });

            Entity {
                heights: column_heights.map(|maybe| maybe.expect("should not be none")),
                schema_type,
            }
        })
        .collect_vec()
}

#[derive(Debug, Eq, PartialEq)]
struct Entity {
    heights: [usize; 5],
    schema_type: SchemaType,
}

#[derive(Debug, Eq, PartialEq)]
enum SchemaType {
    Lock,
    Key,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
#####
.####
.####
.####
.#.#.
.#...
.....

#####
##.##
.#.##
...##
...#.
...#.
.....

.....
#....
#....
#...#
#.#.#
#.###
#####

.....
.....
#.#..
###..
###.#
###.#
#####

.....
.....
.....
#....
#.#..
#.#.#
#####
        "#
        .trim();
        assert_eq!("3", process(input)?);
        Ok(())
    }

    #[test]
    fn test_lock_parsing() {
        let input = r#"
#####
.####
.####
.####
.#.#.
.#...
.....
"#
        .trim();
        let actual = parse(input);

        assert_eq!(
            actual,
            vec![Entity {
                schema_type: SchemaType::Lock,
                heights: [0, 5, 3, 4, 3]
            }]
        );
    }

    #[test]
    fn test_key_parsing() {
        let input = r#"
.....
#....
#....
#...#
#.#.#
#.###
#####
"#
        .trim();
        let actual = parse(input);

        assert_eq!(
            actual,
            vec![Entity {
                schema_type: SchemaType::Key,
                heights: [5, 0, 2, 1, 3]
            }]
        );
    }
}
