use glam::IVec2;
use itertools::Itertools;
use pathfinding::prelude::astar_bag;
use std::collections::HashMap;

#[tracing::instrument]
pub fn process(_input: &str) -> miette::Result<String> {
    todo!("day 01 - part 1");
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

fn compute_all_sequences_for_robot(input: &str, key_map: &KeyMap) -> Vec<String> {
    // robot arm starts on A
    let sequences = ("A".to_owned() + input)
        .chars()
        .tuple_windows()
        .map(|(from, to)| {
            compute_optimal_moves_for_robot(from, to, key_map)
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

    dbg!(sequences);
    dbg!(&all_possible_sequences);

    all_possible_sequences
}

fn compute_optimal_moves_for_robot(from: char, to: char, key_map: &KeyMap) -> Vec<String> {
    let start: IVec2 = key_map.char_to_loc[&from];
    let destination: IVec2 = key_map.char_to_loc[&to];

    let (sequences, cost) = astar_bag(
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
    use crate::NUMERIC_KEY_MAP;
    use assert_unordered::assert_eq_unordered;
    use itertools::Itertools;
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
        todo!();

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

        let actual_sequences: Vec<String> =
            compute_all_sequences_for_robot("029A", &numeric_key_map);

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

        assert_eq_unordered!(
            compute_optimal_moves_for_robot(from, to, &numeric_key_map),
            expected
        );
    }
}
