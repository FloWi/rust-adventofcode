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

fn compute_optimal_sequences_for_robot<'a>(input: &'a str, key_map: &KeyMap) -> Vec<&'a str> {
    let start = key_map.char_to_loc[&'A'];
    let start = key_map.char_to_loc[&'A'];

    //dijkstra_all(start)
    todo!()
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
        #[case] example_sequence: &str,
    ) -> miette::Result<()> {
        assert_eq!(input.len(), example_sequence.len());

        Ok(())
    }

    #[test]
    fn first_robot_moves() {
        let input = "029A";
        let numeric_key_map = KeyMap::new(&NUMERIC_KEY_MAP);
        let expected_optimal_sequences = vec!["<A^A>^^AvvvA", "<A^A^>^AvvvA", "<A^A^^>AvvvA"];

        let actual_sequences: Vec<&str> =
            compute_optimal_sequences_for_robot("029A", &numeric_key_map);

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

        /*

        8



        9
             */

        assert_eq_unordered!(
            compute_optimal_moves_for_robot(from, to, &numeric_key_map),
            expected
        );
    }
}
