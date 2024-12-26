use glam::IVec2;
use itertools::Itertools;
use pathfinding::prelude::astar_bag;
use std::collections::HashMap;
use tracing::debug;

pub mod part1;
pub mod part2;

/*
+---+---+---+
| 7 | 8 | 9 |
+---+---+---+
| 4 | 5 | 6 |
+---+---+---+
| 1 | 2 | 3 |
+---+---+---+
    | 0 | A |
    +---+---+
 */
const NUMERIC_KEY_MAP: [(IVec2, char); 11] = [
    (IVec2::new(0, 0), '7'),
    (IVec2::new(1, 0), '8'),
    (IVec2::new(2, 0), '9'),
    (IVec2::new(0, 1), '4'),
    (IVec2::new(1, 1), '5'),
    (IVec2::new(2, 1), '6'),
    (IVec2::new(0, 2), '1'),
    (IVec2::new(1, 2), '2'),
    (IVec2::new(2, 2), '3'),
    (IVec2::new(1, 3), '0'),
    (IVec2::new(2, 3), 'A'),
];

const DIRECTION_KEY_MAP: [(IVec2, char); 5] = [
    (IVec2::new(1, 0), '^'),
    (IVec2::new(2, 0), 'A'),
    (IVec2::new(0, 1), '<'),
    (IVec2::new(1, 1), 'v'),
    (IVec2::new(2, 1), '>'),
];

fn compute_complexity(input_code: &str, level: u32) -> usize {
    let numeric_key_map = KeyMap::new(&NUMERIC_KEY_MAP);
    let directional_key_map = KeyMap::new(&DIRECTION_KEY_MAP);

    let shortest_length = compute_number_of_sequences_str(
        input_code,
        level,
        level,
        &numeric_key_map,
        &directional_key_map,
        &mut HashMap::new(),
    );

    let numeric_part_of_code = input_code
        .strip_suffix("A")
        .unwrap()
        .parse::<usize>()
        .unwrap();

    shortest_length * numeric_part_of_code
}

struct KeyMap {
    char_to_loc: HashMap<char, IVec2>,
    loc_to_char: HashMap<IVec2, char>,
}

const NEIGHBOR_DIRECTIONS: [IVec2; 4] = [IVec2::X, IVec2::Y, IVec2::NEG_X, IVec2::NEG_Y];

impl KeyMap {
    fn new(input: &[(IVec2, char)]) -> Self {
        let char_to_loc: HashMap<char, IVec2> =
            input.iter().map(|(loc, char)| (*char, *loc)).collect();
        Self {
            char_to_loc,
            loc_to_char: input.iter().cloned().collect(),
        }
    }
}

/*
There are many shortest possible sequences of directional keypad button presses that would
cause this robot
to tell the second robot
to tell the first robot
to eventually type 029A on the door.

One such sequence is <vA<AA>>^AvAA<^A>A<v<A>>^AvA^A<vA>^A<v<A>^A>AAvA^A<v<A>A>^AAAvA<^A>A.

In summary, there are the following keypads:

One directional keypad that you are using.
Two directional keypads that robots are using.
One numeric keypad (on a door) that a robot is using.


<vA<AA>>^AvAA<^A>A<v<A>>^AvA^A<vA>^A<v<A>^A>AAvA^A<v<A>A>^AAAvA<^A>A
v<<A>>^A<A>AvA<^AA>A<vAAA>^A
<A^A>^^AvvvA
029A

 */

fn compute_optimal_moves_for_robot(
    from: char,
    to: char,
    key_map: &KeyMap,
    level: u32,
    call_map: &mut HashMap<(char, char, u32), usize>,
) -> Vec<String> {
    let start: IVec2 = key_map.char_to_loc[&from];
    let destination: IVec2 = key_map.char_to_loc[&to];

    call_map
        .entry((from, to, level))
        .and_modify(|counter| *counter += 1)
        .or_insert(1);

    let (sequences, _cost) = astar_bag(
        &start,
        |pos| {
            NEIGHBOR_DIRECTIONS
                .into_iter()
                .filter_map(move |offset| {
                    let neighbor_pos = pos + offset;
                    key_map
                        .loc_to_char
                        .contains_key(&neighbor_pos)
                        .then_some((neighbor_pos, 1))
                })
                .collect_vec()
        },
        |_| 1,
        |pos| pos == &destination,
    )
    .unwrap();

    let optimal_sequences = sequences
        .map(|positions| {
            let movement_str = positions
                .iter()
                .tuple_windows()
                .map(|(from_pos, to_pos)| {
                    let diff = to_pos - from_pos;
                    match diff {
                        IVec2 { x, y } if x == -1 && y == 0 => '<',
                        IVec2 { x, y } if x == 1 && y == 0 => '>',
                        IVec2 { x, y } if x == 0 && y == -1 => '^',
                        IVec2 { x, y } if x == 0 && y == 1 => 'v',
                        _ => panic!("should not happen"),
                    }
                })
                .join("");
            format!("{movement_str}A")
        })
        .collect_vec();
    optimal_sequences
}

fn compute_number_of_sequences_str(
    input: &str,
    level: u32,
    max_level: u32,
    numeric_keypad: &KeyMap,
    directional_keypad: &KeyMap,
    cache: &mut HashMap<(char, char, u32), usize>,
) -> usize {
    format!("A{input}")
        .chars()
        .tuple_windows()
        .map(|(from, to)| {
            compute_shortest_sequence_length_level_0(
                from,
                to,
                level,
                max_level,
                numeric_keypad,
                directional_keypad,
                cache,
            )
        })
        .sum()
}

fn compute_shortest_sequence_length_level_0(
    from: char,
    to: char,
    level: u32,
    max_level: u32,
    numeric_keypad: &KeyMap,
    directional_keypad: &KeyMap,
    cache: &mut HashMap<(char, char, u32), usize>,
) -> usize {
    let keypad = match level {
        l if l == max_level => numeric_keypad,
        _ => directional_keypad,
    };
    debug!("computing: from = {from}, to = {to}, level = {level}, max_level = {max_level}");

    match cache.get(&(from, to, level)) {
        Some(result) => {
            debug!("cache_hit: from = {from}, to = {to}, level = {level}, max_level = {max_level}, result = {result}");

            *result
        }
        None => {
            let moves =
                compute_optimal_moves_for_robot(from, to, keypad, level, &mut HashMap::new());

            let result = if level == 0 {
                moves[0].len()
            } else {
                let next_level = level - 1;
                debug!(
                    "recursing to level {next_level} for moves: {}",
                    &moves.join(", ")
                );
                moves
                    .iter()
                    .map(|seq_for_next_robot| {
                        compute_number_of_sequences_str(
                            seq_for_next_robot.as_str(),
                            next_level,
                            max_level,
                            numeric_keypad,
                            directional_keypad,
                            cache,
                        )
                    })
                    .min()
                    .unwrap()
            };
            debug!("computed result: from = {from}, to = {to}, level = {level}, max_level = {max_level}, moves: {}, result = {result}", &moves.join(", "));
            cache.insert((from, to, level), result);
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DIRECTION_KEY_MAP, NUMERIC_KEY_MAP};
    use assert_unordered::assert_eq_unordered;
    use itertools::Itertools;
    use nom::Parser;
    use rstest::rstest;

    #[test_log::test]
    fn test_complexity_first_input() -> miette::Result<()> {
        assert_eq!(compute_complexity("029A", 2), 1972);

        Ok(())
    }

    /*
    029A: <vA<AA>>^AvAA<^A>A<v<A>>^AvA^A<vA>^A<v<A>^A>AAvA^A<v<A>A>^AAAvA<^A>A
    980A: <v<A>>^AAAvA^A<vA<AA>>^AvAA<^A>A<v<A>A>^AAAvA<^A>A<vA>^A<A>A
    179A: <v<A>>^A<vA<A>>^AAvAA<^A>A<v<A>>^AAvA^A<vA>^AA<A>A<v<A>A>^AAAvA<^A>A
    456A: <v<A>>^AA<vA<A>>^AAvAA<^A>A<vA>^A<A>A<vA>^A<A>A<v<A>A>^AAvA<^A>A
    379A: <v<A>>^AvA^A<vA<AA>>^AAvA<^A>AAvA^A<vA>^AA<A>A<v<A>A>^AAAvA<^A>A
         */
    #[rstest]
    #[case(
        "029A",
        "<vA<AA>>^AvAA<^A>A<v<A>>^AvA^A<vA>^A<v<A>^A>AAvA^A<v<A>A>^AAAvA<^A>A"
    )]
    #[case("980A", "<v<A>>^AAAvA^A<vA<AA>>^AvAA<^A>A<v<A>A>^AAAvA<^A>A<vA>^A<A>A")]
    #[case(
        "179A",
        "<v<A>>^A<vA<A>>^AAvAA<^A>A<v<A>>^AAvA^A<vA>^AA<A>A<v<A>A>^AAAvA<^A>A"
    )]
    #[case(
        "456A",
        "<v<A>>^AA<vA<A>>^AAvAA<^A>A<vA>^A<A>A<vA>^A<A>A<v<A>A>^AAvA<^A>A"
    )]
    #[case(
        "379A",
        "<v<A>>^AvA^A<vA<AA>>^AAvA<^A>AAvA^A<vA>^AA<A>A<v<A>A>^AAAvA<^A>A"
    )]
    fn test_shortest_sequence(
        #[case] input: &str,
        #[case] one_example_sequence: &str,
    ) -> miette::Result<()> {
        Ok(())
    }

    #[rstest]
    #[case('A', '0', "<A")]
    #[case('A', '1', "^<<A, <^<A")]
    #[case('A', '2', "^<A, <^A")]
    #[case('A', '3', "^A")]
    #[case('A', '4', "^^<<A, <^^<A, ^<^<A, <^<^A, ^<<^A")]
    #[case('A', '5', "^^<A, <^^A, ^<^A")]
    #[case('A', '6', "^^A")]
    #[case(
        'A',
        '7',
        "^^<^<A, ^<^^<A, <^^^<A, ^^^<<A, ^^<<^A, ^<^<^A, <^^<^A, ^<<^^A, <^<^^A"
    )]
    #[case('A', '8', "^^^<A, ^^<^A, ^<^^A, <^^^A")]
    #[case('A', '9', "^^^A")]
    fn numeric_robot_one_move(
        #[case] from: char,
        #[case] to: char,
        #[case] expected_move_sequence_csv: &str,
    ) {
        let numeric_key_map = KeyMap::new(&NUMERIC_KEY_MAP);

        let expected = expected_move_sequence_csv
            .split(", ")
            .map(|sub| sub.to_string())
            .collect_vec();

        let mut call_map = HashMap::new();

        assert_eq_unordered!(
            compute_optimal_moves_for_robot(from, to, &numeric_key_map, 1, &mut call_map),
            expected
        );
    }

    #[test]
    fn refactoring_to_dfs_level_0_1_char() {
        let numeric_key_map = KeyMap::new(&NUMERIC_KEY_MAP);
        let directional_key_map = KeyMap::new(&DIRECTION_KEY_MAP);

        let mut cache = HashMap::new();

        let actual = compute_number_of_sequences_str(
            "0",
            0,
            0,
            &numeric_key_map,
            &directional_key_map,
            &mut cache,
        );

        assert_eq!(actual, 2);
    }

    #[test]
    fn refactoring_to_dfs_level_0__2_chars() {
        let numeric_key_map = KeyMap::new(&NUMERIC_KEY_MAP);
        let directional_key_map = KeyMap::new(&DIRECTION_KEY_MAP);

        let mut cache = HashMap::new();

        let actual = compute_number_of_sequences_str(
            "02",
            0,
            0,
            &numeric_key_map,
            &directional_key_map,
            &mut cache,
        );

        assert_eq!(actual, 4);
    }

    #[test]
    fn refactoring_to_dfs_level_0__3_chars() {
        let numeric_key_map = KeyMap::new(&NUMERIC_KEY_MAP);
        let directional_key_map = KeyMap::new(&DIRECTION_KEY_MAP);

        let mut cache = HashMap::new();

        let actual = compute_number_of_sequences_str(
            "029",
            0,
            0,
            &numeric_key_map,
            &directional_key_map,
            &mut cache,
        );

        assert_eq!(actual, 8);
    }

    #[test]
    fn refactoring_to_dfs_level_0__all_4_chars() {
        let numeric_key_map = KeyMap::new(&NUMERIC_KEY_MAP);
        let directional_key_map = KeyMap::new(&DIRECTION_KEY_MAP);

        let mut cache = HashMap::new();

        let actual = compute_number_of_sequences_str(
            "029A",
            0,
            0,
            &numeric_key_map,
            &directional_key_map,
            &mut cache,
        );

        assert_eq!(actual, 12);
    }

    #[test]
    fn refactoring_to_dfs_level_1_1_char() {
        let numeric_key_map = KeyMap::new(&NUMERIC_KEY_MAP);
        let directional_key_map = KeyMap::new(&DIRECTION_KEY_MAP);

        let mut cache = HashMap::new();

        let actual = compute_number_of_sequences_str(
            "0",
            1,
            1,
            &numeric_key_map,
            &directional_key_map,
            &mut cache,
        );

        assert_eq!(actual, 8);
    }

    #[test]
    fn refactoring_to_dfs_level_1_2_chars() {
        let numeric_key_map = KeyMap::new(&NUMERIC_KEY_MAP);
        let directional_key_map = KeyMap::new(&DIRECTION_KEY_MAP);

        let mut cache = HashMap::new();

        let actual = compute_number_of_sequences_str(
            "02",
            1,
            1,
            &numeric_key_map,
            &directional_key_map,
            &mut cache,
        );

        assert_eq!(actual, 12);
    }

    #[test]
    fn refactoring_to_dfs_level_1_3_chars() {
        let numeric_key_map = KeyMap::new(&NUMERIC_KEY_MAP);
        let directional_key_map = KeyMap::new(&DIRECTION_KEY_MAP);

        let mut cache = HashMap::new();

        let actual = compute_number_of_sequences_str(
            "029",
            1,
            1,
            &numeric_key_map,
            &directional_key_map,
            &mut cache,
        );

        assert_eq!(actual, 20);
    }

    #[test]
    fn refactoring_to_dfs_level_1_all_4_chars() {
        let numeric_key_map = KeyMap::new(&NUMERIC_KEY_MAP);
        let directional_key_map = KeyMap::new(&DIRECTION_KEY_MAP);

        let mut cache = HashMap::new();

        let actual = compute_number_of_sequences_str(
            "029A",
            1,
            1,
            &numeric_key_map,
            &directional_key_map,
            &mut cache,
        );

        assert_eq!(actual, 28);
    }

    #[test]
    fn refactoring_to_dfs_level_2_all_4_chars() {
        let numeric_key_map = KeyMap::new(&NUMERIC_KEY_MAP);
        let directional_key_map = KeyMap::new(&DIRECTION_KEY_MAP);

        let mut cache = HashMap::new();

        let actual = compute_number_of_sequences_str(
            "029A",
            2,
            2,
            &numeric_key_map,
            &directional_key_map,
            &mut cache,
        );
        // <vA<AA>>^AvAA<^A>A<v<A>>^AvA^A<vA>^A<v<A>^A>AAvA^A<v<A>A>^AAAvA<^A>A

        assert_eq!(actual, 68);
    }
}
