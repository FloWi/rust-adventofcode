use crate::part1::MoveResult::{PlayerMovedToEmptySpot, Problem, UnableToMove};
use glam::IVec2;
use itertools::Either::{Left, Right};
use itertools::{Either, Itertools};
use miette::miette;
use nom::character::complete::{line_ending, one_of};
use nom::multi::{many1, separated_list1};
use nom::sequence::{separated_pair, tuple};
use nom::IResult;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::iter::successors;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, warehouse) = parse(input).map_err(|e| miette!("parse failed {}", e))?;

    dbg!(warehouse);

    let result = 42;

    Ok(result.to_string())
}

#[derive(Debug)]
enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Clone, Debug, PartialEq)]
enum Tile {
    Empty,
    Wall,
    Box,
}

#[derive(PartialEq)]
struct Player;

#[derive(Debug)]
struct Warehouse {
    game_map: HashMap<IVec2, Tile>,
    movement_sequence: Vec<Direction>,
    map_dimensions: IVec2,
    player_location: IVec2,
}

fn move_player(
    game_map: &mut HashMap<IVec2, Tile>,
    direction: Direction,
    map_dimensions: IVec2,
    player_location: IVec2,
) -> MoveResult {
    let x_range = 0..map_dimensions.x;
    let y_range = 0..map_dimensions.y;

    let offset: IVec2 = match direction {
        Direction::North => IVec2::NEG_Y,
        Direction::East => IVec2::X,
        Direction::South => IVec2::Y,
        Direction::West => IVec2::NEG_X,
    };
    let locations_affected_by_move = successors(Some(player_location.clone()), |pos| {
        let new_pos = pos + offset;
        let yield_result =
            (x_range.contains(&new_pos.x) && y_range.contains(&new_pos.y)).then_some(new_pos);
        yield_result
    })
    .skip(1)
    .collect_vec();

    dbg!(&locations_affected_by_move);

    let tiles_and_locations_affected_by_move = locations_affected_by_move
        .into_iter()
        .map(|loc| (loc, game_map.get(&loc).unwrap()))
        .collect_vec();

    dbg!(&tiles_and_locations_affected_by_move);

    match tiles_and_locations_affected_by_move.first() {
        None => Problem("No move left (shouldn't happen)"),
        Some((_pos, Tile::Wall)) => UnableToMove(MoveProblem::PlayerDirectlyBlockedByWall),
        Some((pos, Tile::Empty)) => PlayerMovedToEmptySpot(*pos),

        Some((_pos, Tile::Box)) => Problem("Player is facing box - TBD"),
    }
}

#[derive(Debug, PartialEq)]
enum MoveProblem {
    PlayerDirectlyBlockedByWall,
    NoSpaceToPushBoxes,
}

#[derive(Debug, PartialEq)]
enum MoveResult<'a> {
    Problem(&'a str),
    UnableToMove(MoveProblem),
    PlayerMovedToEmptySpot(IVec2),
    PlayerPushedBoxes(IVec2),
}

fn parse(input: &str) -> IResult<&str, Warehouse> {
    let (rest, (game_map_chars, moves)): (&str, (Vec<Vec<char>>, Vec<Vec<char>>)) = separated_pair(
        separated_list1(line_ending, many1(one_of("#.@O"))),
        tuple((line_ending, line_ending)),
        separated_list1(line_ending, many1(one_of("^>v<"))),
    )(input)?;

    let game_map_with_player: HashMap<IVec2, Either<Player, Tile>> = game_map_chars
        .iter()
        .enumerate()
        .flat_map(|(y, row)| {
            row.iter().enumerate().map(move |(x, char)| {
                let tile = match *char {
                    '.' => Right(Tile::Empty),
                    'O' => Right(Tile::Box),
                    '#' => Right(Tile::Wall),
                    '@' => Left(Player),
                    unknown => {
                        panic!("Tile '{unknown}' is unknown. Should not happen")
                    }
                };
                (IVec2::new(x as i32, y as i32), tile)
            })
        })
        .collect();

    let movement_sequence = moves
        .iter()
        .flatten()
        .map(|char| match *char {
            '<' => Direction::West,
            '^' => Direction::North,
            '>' => Direction::East,
            'v' => Direction::South,
            unknown => {
                panic!("Direction '{unknown}' is unknown. Should not happen")
            }
        })
        .collect_vec();

    let map_dimensions = *game_map_with_player
        .keys()
        .max_by_key(|IVec2 { x, y }| (x, y))
        .unwrap()
        + IVec2::new(1, 1);

    let player_location = *game_map_with_player
        .iter()
        .find(|(pos, tile)| **tile == Left(Player))
        .unwrap()
        .0;

    //create game_map with tiles only (player is removed)
    let game_map: HashMap<IVec2, Tile> = game_map_with_player
        .into_iter()
        .map(|(pos, either_player_or_tile)| {
            (
                pos,
                match either_player_or_tile {
                    Left(Player) => Tile::Empty,
                    Right(tile) => tile,
                },
            )
        })
        .collect();

    Ok((
        rest,
        Warehouse {
            game_map,
            movement_sequence,
            map_dimensions,
            player_location,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
########
#..O.O.#
##@.O..#
#...O..#
#.#.O..#
#...O..#
#......#
########

<^^>>>vv
<v>>v<<
        "#
        // added newline to movement sequence to be sure I handle it correctly
        .trim();
        assert_eq!("2028", process(input)?);
        Ok(())
    }

    #[test]
    fn test_process_larger_example() -> miette::Result<()> {
        let input = r#"
##########
#..O..O.O#
#......O.#
#.OO..O.O#
#..O@..O.#
#O#..O...#
#O..O..O.#
#.OO.O.OO#
#....O...#
##########

<vv>^<v^>v>^vv^v>v<>v^v<v<^vv<<<^><<><>>v<vvv<>^v^>^<<<><<v<<<v^vv^v>^
vvv<<^>^v^^><<>>><>^<<><^vv^^<>vvv<>><^^v>^>vv<>v<<<<v<^v>^<^^>>>^<v<v
><>vv>v^v^<>><>>>><^^>vv>v<^^^>>v^v^<^^>v^^>v^<^v>v<>>v^v^<v>v^^<^^vv<
<<v<^>>^^^^>>>v^<>vvv^><v<<<>^^^vv^<vvv>^>v<^^^^v<>^>vvvv><>>v^<<^^^^^
^><^><>>><>^^<<^^v>>><^<v>^<vv>>v>>>^v><>^v><<<<v>>v<v<v>vvv>^<><<>^><
^>><>^v<><^vvv<^^<><v<<<<<><^v<<<><<<^^<v<^^^><^>>^<v^><<<^>>^v<v^v<v^
>^>>^v>vv>^<<^v<>><<><<v<<v><>v<^vv<<<>^^v^>^^>>><<^v>>v^v><^^>>^<>vv^
<><^^>^^^<><vvvvv^v<v<<>^v<v>v<<^><<><<><<<^^<<<^<<>><<><^^^>^^<>^>v<>
^^>vv<^v^v<vv>^<><v<^v>^^^>>>^^vvv^>vvv<>>>^<^>>>>>^<<^v>^vvv<>^<><<v>
v^^>>><<^^<>>^v^<v^vv<>v^<<>^<^v^v><^<<<><<^<v><v<>vv>>v><v^<vv<>v^<<^
        "#
        // added newline to movement sequence to be sure I handle it correctly
        .trim();
        assert_eq!("10092", process(input)?);
        Ok(())
    }

    fn get_small_example_warehouse() -> Warehouse {
        let input = r#"
########
#..O.O.#
##@.O..#
#...O..#
#.#.O..#
#...O..#
#......#
########

<^^>>>vv
<v>>v<<
        "#
        // added newline to movement sequence to be sure I handle it correctly
        .trim();

        let (_, warehouse) = parse(input).unwrap();
        warehouse
    }

    #[test]
    fn test_movement_west() -> miette::Result<()> {
        let Warehouse {
            game_map: original_game_map,
            movement_sequence,
            map_dimensions,
            player_location,
        } = get_small_example_warehouse();
        let mut game_map = original_game_map.clone();

        assert_eq!(
            move_player(
                &mut game_map,
                Direction::West,
                map_dimensions,
                player_location,
            ),
            UnableToMove(MoveProblem::PlayerDirectlyBlockedByWall)
        );
        assert_eq!(game_map, original_game_map);
        Ok(())
    }

    #[test]
    fn test_movement_east_1x() -> miette::Result<()> {
        let Warehouse {
            game_map: original_game_map,
            movement_sequence,
            map_dimensions,
            player_location,
        } = get_small_example_warehouse();

        let mut game_map = original_game_map.clone();

        assert_eq!(
            move_player(
                &mut game_map,
                Direction::East,
                map_dimensions,
                player_location,
            ),
            PlayerMovedToEmptySpot(player_location + IVec2::X)
        );
        assert_eq!(game_map, original_game_map);
        Ok(())
    }

    #[test]
    fn test_movement_east_2x_push_box() -> miette::Result<()> {
        let Warehouse {
            game_map: original_game_map,
            movement_sequence,
            map_dimensions,
            player_location,
        } = get_small_example_warehouse();

        let mut game_map = original_game_map.clone();

        //2nd move east will push one box
        let pos_after_1st_move = player_location + IVec2::X;
        assert_eq!(
            move_player(
                &mut game_map,
                Direction::East,
                map_dimensions,
                player_location,
            ),
            PlayerMovedToEmptySpot(pos_after_1st_move)
        );

        //2nd move east will push one box
        assert_eq!(
            move_player(
                &mut game_map,
                Direction::East,
                map_dimensions,
                pos_after_1st_move,
            ),
            PlayerMovedToEmptySpot(pos_after_1st_move + IVec2::X)
        );
        assert_eq!(game_map, original_game_map);

        Ok(())
    }
}
