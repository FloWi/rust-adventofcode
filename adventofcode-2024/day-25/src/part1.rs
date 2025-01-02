use itertools::Itertools;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let keys_and_locks = parse(input);
    todo!("day 01 - part 1");
}

fn parse(input: &str) -> Vec<ScannedThing> {
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

            let mut column_heights: [Option<usize>; 5] = [None, None, None, None, None];

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
                            column_heights[x] = Some(5 - y)
                        }
                    }
                })
            });

            match schema_type {
                SchemaType::Lock => ScannedThing::Lock {
                    heights: column_heights.map(|maybe| maybe.expect("should not be none")),
                },
                SchemaType::Key => ScannedThing::Key {
                    heights: column_heights.map(|maybe| maybe.expect("should not be none")),
                },
            }
        })
        .collect_vec()
}

#[derive(Debug, Eq, PartialEq)]
enum ScannedThing {
    Key { heights: [usize; 5] },
    Lock { heights: [usize; 5] },
}

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
        assert_eq!("", process(input)?);
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
            vec![ScannedThing::Lock {
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
            vec![ScannedThing::Key {
                heights: [5, 0, 2, 1, 3]
            }]
        );
    }
}
