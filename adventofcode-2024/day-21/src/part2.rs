use crate::{DIRECTION_KEY_MAP, NUMERIC_KEY_MAP};
use glam::IVec2;
use itertools::Itertools;
use pathfinding::prelude::astar_bag;
use std::cmp::max;
use std::collections::HashMap;
use tracing::info;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let codes = input.trim().lines().collect_vec();

    let mut call_map = HashMap::new();

    let complexities = codes
        .iter()
        .cloned()
        .map(|code| compute_complexity(code, &mut call_map))
        .collect_vec();

    let result: usize = complexities.iter().sum();

    Ok(result.to_string())
}

fn compute_complexity(input_code: &str, call_map: &mut HashMap<(char, char, u32), usize>) -> usize {
    let numeric_key_map = KeyMap::new(&NUMERIC_KEY_MAP);
    let directional_key_map = KeyMap::new(&DIRECTION_KEY_MAP);

    // final numeric robot
    let robot_0_sequences =
        compute_all_sequences_for_robot(input_code, &numeric_key_map, 0, call_map);

    // 1st directional robot
    let robot_1_sequences = robot_0_sequences
        .into_iter()
        .flat_map(|seq| compute_all_sequences_for_robot(&seq, &directional_key_map, 1, call_map))
        .collect_vec();

    let shortest_length_robot_1_sequences =
        robot_1_sequences.iter().map(|seq| seq.len()).min().unwrap();

    let robot_1_sequences = robot_1_sequences
        .iter()
        .filter(|seq| seq.len() == shortest_length_robot_1_sequences)
        .collect_vec();

    // 2nd directional robot
    let robot_2_sequences = robot_1_sequences
        .into_iter()
        .flat_map(|seq| compute_all_sequences_for_robot(&seq, &directional_key_map, 2, call_map))
        .collect_vec();

    let shortest_length_robot_2_sequences =
        robot_2_sequences.iter().map(|seq| seq.len()).min().unwrap();
    // let robot_2_sequences = robot_2_sequences.iter().filter(|seq| seq.len() == shortest_length_robot_2_sequences).collect_vec();

    let numeric_part_of_code = input_code
        .strip_suffix("A")
        .unwrap()
        .parse::<usize>()
        .unwrap();

    shortest_length_robot_2_sequences * numeric_part_of_code
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

fn compute_all_sequences_for_robot(
    input: &str,
    key_map: &KeyMap,
    level: u32,
    call_map: &mut HashMap<(char, char, u32), usize>,
) -> Vec<String> {
    // robot arm starts on A
    let sequences = ("A".to_owned() + input)
        .chars()
        .tuple_windows()
        .map(|(from, to)| {
            compute_optimal_moves_for_robot(from, to, key_map, level, call_map)
                .into_iter()
                .map(|movement_sequence| movement_sequence + "A")
                .collect_vec()
        })
        .collect_vec();

    let all_possible_sequences = sequences
        .iter()
        .cloned()
        .multi_cartesian_product()
        .map(|combo| combo.join(""))
        .collect_vec();

    /*
    [day-21/src/part1.rs:64:5] sequences = [
        [
            "<A",
        ],
        [
            "^A",
        ],
        [
            "^^>A",
            ">^^A",
            "^>^A",
        ],
        [
            "vvvA",
        ],
    ]

    [day-21/src/part1.rs:85:5] all_possible_sequences = [
    "<A^A^>^AvvvA",
    "<A^A>^^AvvvA",
    "<A^A^^>AvvvA",
    ]
             */

    let shortest_length = all_possible_sequences
        .iter()
        .map(|seq| seq.len())
        .min()
        .unwrap();
    assert!(
        all_possible_sequences
            .iter()
            .all(|seq| seq.len() == shortest_length),
        "all sequences must have the same length"
    );

    all_possible_sequences
}

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
            positions
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
                .join("")
        })
        .collect_vec();
    optimal_sequences
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DIRECTION_KEY_MAP, NUMERIC_KEY_MAP};
    use assert_unordered::assert_eq_unordered;
    use itertools::Itertools;
    use nom::Parser;
    use rstest::rstest;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
029A
980A
179A
456A
379A
        "#
        .trim();
        assert_eq!("126384", process(input)?);
        Ok(())
    }

    #[test_log::test]
    fn test_complexity_first_input() -> miette::Result<()> {
        let mut call_map = HashMap::new();
        assert_eq!(compute_complexity("029A", &mut call_map), 1972);

        dbg!(call_map);
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

    #[test]
    fn first_robot_moves() {
        let input = "029A";
        let numeric_key_map = KeyMap::new(&NUMERIC_KEY_MAP);
        let expected_optimal_sequences = vec!["<A^A>^^AvvvA", "<A^A^>^AvvvA", "<A^A^^>AvvvA"]
            .into_iter()
            .map(|string| string.to_string())
            .collect_vec();

        let mut call_map = HashMap::new();

        let actual_sequences: Vec<String> =
            compute_all_sequences_for_robot("029A", &numeric_key_map, 1, &mut call_map);

        assert_eq_unordered!(actual_sequences, expected_optimal_sequences)
    }

    #[rstest]
    #[case('A', '0', "<")]
    #[case('A', '1', "^<<, <^<")]
    #[case('A', '2', "^<, <^")]
    #[case('A', '3', "^")]
    #[case('A', '4', "^^<<, <^^<, ^<^<, <^<^, ^<<^")]
    #[case('A', '5', "^^<, <^^, ^<^")]
    #[case('A', '6', "^^")]
    #[case(
        'A',
        '7',
        "^^<^<, ^<^^<, <^^^<, ^^^<<, ^^<<^, ^<^<^, <^^<^, ^<<^^, <^<^^"
    )]
    #[case('A', '8', "^^^<, ^^<^, ^<^^, <^^^")]
    #[case('A', '9', "^^^")]
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
    fn refactoring_to_dfs() {
        let test_code = "029A";

        compute_number_of_sequences_improved(test_code, 2);
    }
}

fn compute_number_of_sequences_improved(input: &str, number_of_numeric_keypads: u32) {
    let numeric_key_map = KeyMap::new(&NUMERIC_KEY_MAP);
    let directional_key_map = KeyMap::new(&DIRECTION_KEY_MAP);

    let mut cache = HashMap::new();

    format!("A{input}")
        .chars()
        .tuple_windows()
        .for_each(|(from, to)| {
            let result = compute_number_of_sequences_recursive(
                from,
                to,
                0,
                number_of_numeric_keypads,
                &numeric_key_map,
                &directional_key_map,
                &mut cache,
            );
            dbg!(result);
        });

    println!("Cache overview: ");
    cache
        .into_iter()
        .sorted_by_key(|((_, _, level), _)| *level)
        .for_each(|((from, to, level), count)| {
            println!("Level {level}: {from}{to} = {count}");
        });

    println!("")
    // dbg!(cache);
}

fn compute_number_of_sequences_recursive(
    from: char,
    to: char,
    level: u32,
    max_level: u32,
    numeric_keypad: &KeyMap,
    directional_keypad: &KeyMap,
    cache: &mut HashMap<(char, char, u32), usize>,
) -> usize {
    match cache.get(&(from, to, level)) {
        Some(&result) => result,
        None => {
            let result = (level..=max_level)
                .map(|current_level| {
                    let keypad = match level {
                        0 => numeric_keypad,
                        n if (1..=max_level).contains(&n) => directional_keypad,
                        _ => panic!("too deep"),
                    };

                    let sequences_for_this_level = compute_optimal_moves_for_robot(
                        from,
                        to,
                        keypad,
                        current_level,
                        &mut HashMap::new(),
                    );

                    if current_level == max_level {
                        let result = sequences_for_this_level.first().unwrap().len();
                        cache.insert((from, to, level), result);
                        return result;
                    } else {
                        let next_level = level + 1;

                        let new_sequences = sequences_for_this_level
                            .iter()
                            .map(|seq| {
                                seq.chars()
                                    .tuple_windows()
                                    .map(|(from_1, to_1)| {
                                        compute_number_of_sequences_recursive(
                                            from_1,
                                            to_1,
                                            next_level,
                                            max_level,
                                            numeric_keypad,
                                            directional_keypad,
                                            cache,
                                        )
                                    })
                                    .sum::<usize>()
                            })
                            .collect_vec();

                        dbg!(next_level, new_sequences);
                    }

                    dbg!(current_level, from, to, &sequences_for_this_level);
                    sequences_for_this_level
                        .iter()
                        .map(|seq| seq.len())
                        .min()
                        .unwrap()
                })
                .sum::<usize>();

            cache.insert((from, to, level), result);
            result
        }
    }
}
