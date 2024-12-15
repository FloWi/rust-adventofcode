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
use tracing::info;
use MoveResult::{PlayerMovedToEmptySpot, PlayerPushedBoxes, UnableToMove};

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (
        _,
        Warehouse {
            mut game_map,
            movement_sequence,
            map_dimensions,
            player_location: original_player_location,
        },
    ) = parse(input).map_err(|e| miette!("parse failed {}", e))?;

    info!(
        "Initial state:\n{}",
        render_map(&game_map, map_dimensions, original_player_location)
    );

    let mut player_location = original_player_location.clone();
    for move_direction in movement_sequence {
        let movement_result = move_player(
            &mut game_map,
            &move_direction,
            map_dimensions,
            player_location,
        )?;
        match movement_result {
            UnableToMove(_) => {}
            PlayerMovedToEmptySpot(new_pos) => player_location = new_pos,
            PlayerPushedBoxes(new_pos) => player_location = new_pos,
        };
        let move_char = match move_direction {
            Direction::West => '<',
            Direction::North => '^',
            Direction::East => '>',
            Direction::South => 'v',
        };

        info!(
            "\nMove {move_char}: \n{}",
            render_map(&game_map, map_dimensions, player_location)
        );
    }
    let result = compute_score(&game_map);

    Ok(result.to_string())
}

fn compute_score(game_map: &HashMap<IVec2, Tile>) -> i32 {
    // The GPS coordinate of a box is equal to
    // 100 times its distance from the top edge of the map
    // plus its distance from the left edge of the map.
    // (This process does not stop at wall tiles; measure all the way to the edges of the map.)
    game_map
        .iter()
        .map(|(pos, tile)| {
            let factor = match *tile {
                Tile::Empty => 0,
                Tile::Wall => 0,
                Tile::Box => 1,
            };
            factor * (pos.y * 100 + pos.x)
        })
        .sum()
}

fn render_map(
    game_map: &HashMap<IVec2, Tile>,
    map_dimensions: IVec2,
    player_location: IVec2,
) -> String {
    (0..map_dimensions.y)
        .map(|y| {
            (0..map_dimensions.x)
                .map(|x| {
                    let pos = IVec2::new(x, y);
                    match &game_map.get(&pos) {
                        None => "",
                        Some(tile) => {
                            let chars = if pos == player_location {
                                "@."
                            } else {
                                match tile {
                                    Tile::Empty => "..",
                                    Tile::Wall => "##",
                                    Tile::Box => "[]",
                                }
                            };
                            chars
                        }
                    }
                })
                .join("")
        })
        .join("\n")
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
    direction: &Direction,
    map_dimensions: IVec2,
    player_location: IVec2,
) -> miette::Result<MoveResult> {
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

    let tiles_and_locations_affected_by_move = locations_affected_by_move
        .into_iter()
        .map(|loc| (loc, game_map.get(&loc).unwrap().clone()))
        .collect_vec();

    match tiles_and_locations_affected_by_move.first() {
        None => miette::bail!("No move left (shouldn't happen)"),
        Some((_pos, Tile::Wall)) => Ok(UnableToMove(MoveProblem::PlayerDirectlyBlockedByWall)),
        Some((pos, Tile::Empty)) => Ok(PlayerMovedToEmptySpot(*pos)),
        Some((pos, Tile::Box)) => {
            // these are the scenarios
            // #..0@   // push one box (move first box to first empty space)
            // #.00@   // push n boxes (move first box to first empty space)
            // #0@..   // can't move, because no space between box and wall
            // #00@.   // can't move, because no space between boxes and wall

            //
            // #..0@   // distance to first empty space: 2; distance to first wall: 4
            // #.00@   // distance to first empty space: 3; distance to first wall: 4
            // #0@..   // distance to first empty space: None; distance to first wall: 2
            // #00@.   // distance to first empty space: None; distance to first wall: 2

            let maybe_first_empty_space = tiles_and_locations_affected_by_move
                .iter()
                .find_map(|(pos, tile)| matches!(tile, Tile::Empty).then_some(pos));

            let first_wall = tiles_and_locations_affected_by_move
                .iter()
                .find_map(|(pos, tile)| matches!(tile, Tile::Wall).then_some(pos))
                .expect("There should _always_ be a wall in line");

            let distance_to_first_wall = player_location.distance_squared(*first_wall);

            let result = match maybe_first_empty_space {
                None => UnableToMove(MoveProblem::NoSpaceToPushBoxes),
                Some(first_empty_space) => {
                    let distance_to_first_empty_space =
                        player_location.distance_squared(*first_empty_space);

                    if distance_to_first_wall < distance_to_first_empty_space {
                        UnableToMove(MoveProblem::NoSpaceToPushBoxes)
                    } else {
                        game_map.insert(*first_empty_space, Tile::Box);
                        game_map.insert(*pos, Tile::Empty);
                        PlayerPushedBoxes(*pos)
                    }
                }
            };

            Ok(result)
        }
    }
}

#[derive(Debug, PartialEq)]
enum MoveProblem {
    PlayerDirectlyBlockedByWall,
    NoSpaceToPushBoxes,
}

#[derive(Debug, PartialEq)]
enum MoveResult {
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

    let scale_factor = IVec2::new(2, 1);

    let map_dimensions = (*game_map_with_player
        .keys()
        .max_by_key(|IVec2 { x, y }| (x, y))
        .unwrap()
        + IVec2::new(1, 1))
        * scale_factor;

    let player_location = *game_map_with_player
        .iter()
        .find(|(pos, tile)| **tile == Left(Player))
        .unwrap()
        .0
        * scale_factor;

    //create game_map with tiles only (player is removed)
    let game_map: HashMap<IVec2, Tile> = game_map_with_player
        .into_iter()
        .map(|(IVec2 { x, y }, either_player_or_tile)| {
            (
                IVec2 { x: x * 2, y },
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

    const LARGER_EXAMPLE_STR: &str = r#"##########
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
v^^>>><<^^<>>^v^<v^vv<>v^<<>^<^v^v><^<<<><<^<v><v<>vv>>v><v^<vv<>v^<<^"#;

    #[test]
    fn test_parsing_and_printing_map() -> miette::Result<()> {
        let input = LARGER_EXAMPLE_STR;

        let (
            _,
            Warehouse {
                game_map,
                movement_sequence,
                map_dimensions,
                player_location,
            },
        ) = parse(input).unwrap();

        let actual_render = render_map(&game_map, map_dimensions, player_location);

        assert_eq!(map_dimensions, IVec2::new(20, 10));
        let expected_render = r#"
####################
##....[]....[]..[]##
##............[]..##
##..[][]....[]..[]##
##....[]@.....[]..##
##[]##....[]......##
##[]....[]....[]..##
##..[][]..[]..[][]##
##........[]......##
####################
        "#
        .trim();

        assert_eq!(actual_render, expected_render);
        Ok(())
    }

    #[test]
    fn test_larger_example_push_box_west() -> miette::Result<()> {
        let input = LARGER_EXAMPLE_STR;

        let (
            _,
            Warehouse {
                game_map: original_game_map,
                movement_sequence,
                map_dimensions,
                player_location,
            },
        ) = parse(input).unwrap();

        let mut game_map = original_game_map.clone();
        let result = move_player(
            &mut game_map,
            &Direction::West,
            map_dimensions,
            player_location,
        )?;

        let actual_render = render_map(&game_map, map_dimensions, player_location);

        let expected_render = r#"
####################
##....[]....[]..[]##
##............[]..##
##..[][]....[]..[]##
##...[]@......[]..##
##[]##....[]......##
##[]....[]....[]..##
##..[][]..[]..[][]##
##........[]......##
####################
        "#
        .trim();

        assert_eq!(actual_render, expected_render);
        Ok(())
    }
}
