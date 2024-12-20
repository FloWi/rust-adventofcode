use crate::part2::MoveProblem::PlayerDirectlyBlockedByWall;
use crate::part2::SingleWidthTile::{BoxClose, BoxOpen};
use glam::IVec2;
use itertools::Either::{Left, Right};
use itertools::{Either, Itertools};
use miette::miette;
use nom::character::complete::{line_ending, one_of};
use nom::multi::{many1, separated_list1};
use nom::sequence::{separated_pair, tuple};
use nom::IResult;
use std::cmp::PartialEq;
use std::collections::{HashMap, VecDeque};
use std::iter::successors;
use tracing::{debug, info};
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

    perform_moves(
        &mut game_map,
        movement_sequence,
        map_dimensions,
        original_player_location,
    )?;
    let result = compute_score(&game_map);

    Ok(result.to_string())
}

fn perform_moves(
    game_map: &mut HashMap<IVec2, SingleWidthTile>,
    movement_sequence: Vec<Direction>,
    map_dimensions: IVec2,
    original_player_location: IVec2,
) -> miette::Result<IVec2> {
    let mut player_location = original_player_location;
    for move_direction in movement_sequence {
        let movement_result =
            move_player(game_map, &move_direction, map_dimensions, player_location)?;
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
            render_map(game_map, map_dimensions, player_location)
        );
    }
    Ok(player_location)
}

fn compute_score(game_map: &HashMap<IVec2, SingleWidthTile>) -> i32 {
    // The GPS coordinate of a box is equal to
    // 100 times its distance from the top edge of the map
    // plus its distance from the left edge of the map.
    // (This process does not stop at wall tiles; measure all the way to the edges of the map.)
    game_map
        .iter()
        .map(|(pos, tile)| {
            let factor = match *tile {
                SingleWidthTile::Wall => 0,
                BoxOpen => 1,
                BoxClose => 0,
            };
            factor * (pos.y * 100 + pos.x)
        })
        .sum()
}

fn render_map(
    game_map: &HashMap<IVec2, SingleWidthTile>,
    map_dimensions: IVec2,
    player_location: IVec2,
) -> String {
    (0..map_dimensions.y)
        .map(|y| {
            (0..map_dimensions.x)
                .map(|x| {
                    let pos = IVec2::new(x, y);
                    match &game_map.get(&pos) {
                        None => {
                            if pos == player_location {
                                "@"
                            } else {
                                "."
                            }
                        }
                        Some(tile) => match tile {
                            SingleWidthTile::Wall => "#",
                            BoxOpen => "[",
                            BoxClose => "]",
                        },
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
enum OriginalTile {
    Empty,
    Wall,
    Box,
}

#[derive(Clone, Debug, PartialEq)]
enum SingleWidthTile {
    Wall,
    BoxOpen,
    BoxClose,
}

impl OriginalTile {
    fn width(&self) -> u32 {
        match self {
            OriginalTile::Empty => 1,
            OriginalTile::Wall => 2,
            OriginalTile::Box => 2,
        }
    }
}

#[derive(PartialEq)]
struct Player;

#[derive(Debug)]
struct Warehouse {
    game_map: HashMap<IVec2, SingleWidthTile>,
    movement_sequence: Vec<Direction>,
    map_dimensions: IVec2,
    player_location: IVec2,
}

fn move_player_horizontally(
    game_map: &mut HashMap<IVec2, SingleWidthTile>,
    direction: &Direction,
    map_dimensions: IVec2,
    player_location: IVec2,
) -> miette::Result<MoveResult> {
    let x_range = 0..map_dimensions.x;

    let offset: IVec2 = match direction {
        Direction::East => IVec2::X,
        Direction::West => IVec2::NEG_X,
        _ => return miette::bail!("Only horizontal moves allowed here."),
    };

    let target_player_position = player_location + offset;

    let locations_affected_by_move = successors(Some(target_player_position), |pos| {
        let new_pos = pos + offset;

        x_range.contains(&new_pos.x).then_some(new_pos)
    })
    .collect_vec();

    let tiles_and_locations_affected_by_move = locations_affected_by_move
        .into_iter()
        .map(|loc| {
            let maybe_tile = game_map.get(&loc).cloned();
            (loc, maybe_tile)
        })
        .collect_vec();

    match tiles_and_locations_affected_by_move.first() {
        None => {
            panic!("shouldn't happen - ran out of space")
        }
        Some((_, None)) => Ok(PlayerMovedToEmptySpot(target_player_position)),

        Some((_loc, Some(tile))) => {
            match tile {
                SingleWidthTile::Wall => Ok(UnableToMove(PlayerDirectlyBlockedByWall)),
                BoxOpen | BoxClose => {
                    // evaluate further
                    // todo!("reached branch {tile:?}");
                    // check first spot after boxes
                    let neighbouring_boxes = tiles_and_locations_affected_by_move
                        .iter()
                        .take_while(|(loc, maybe_tile)| match maybe_tile {
                            None => false,
                            Some(tile) => match tile {
                                SingleWidthTile::Wall => false,
                                BoxOpen => true,
                                BoxClose => true,
                            },
                        })
                        .filter_map(|(loc, maybe_tile)| match maybe_tile {
                            None => None,
                            Some(tile) => match tile {
                                SingleWidthTile::Wall => None,
                                BoxOpen => Some((loc, BoxOpen)),
                                BoxClose => Some((loc, BoxClose)),
                            },
                        })
                        .collect_vec();

                    let maybe_first_tile_after_boxes =
                        tiles_and_locations_affected_by_move.get(neighbouring_boxes.len());
                    match maybe_first_tile_after_boxes {
                        None => {
                            panic!("shouldn't happen - ran out of space")
                        }
                        Some((loc, maybe_tile)) => {
                            match maybe_tile {
                                None => {
                                    // we found an empty slot to push the boxes
                                    for (loc, _) in neighbouring_boxes.iter() {
                                        game_map.remove(loc);
                                    }
                                    for (loc, box_tile) in neighbouring_boxes {
                                        game_map.insert(loc + offset, box_tile);
                                    }

                                    Ok(PlayerPushedBoxes(target_player_position))

                                }
                                Some(SingleWidthTile::Wall) => {
                                    Ok(UnableToMove(MoveProblem::NoSpaceToPushBoxes))
                                }
                                _ =>
                                    panic!("shouldn't happen - {maybe_tile:?} should have been handled earlier")
                            }
                        }
                    }
                }
            }
        }
    }
}

fn move_player_vertically(
    game_map: &mut HashMap<IVec2, SingleWidthTile>,
    direction: &Direction,
    map_dimensions: IVec2,
    player_location: IVec2,
) -> miette::Result<MoveResult> {
    let offset: IVec2 = match direction {
        Direction::North => IVec2::NEG_Y,
        Direction::South => IVec2::Y,
        _ => return miette::bail!("Only horizontal moves allowed here."),
    };

    let new_location = player_location + offset;
    let maybe_tile_at_new_location = game_map.get(&new_location);

    match maybe_tile_at_new_location {
        None => Ok(PlayerMovedToEmptySpot(new_location)),
        Some(SingleWidthTile::Wall) => Ok(UnableToMove(PlayerDirectlyBlockedByWall)),
        Some(BoxOpen | BoxClose) => {
            match find_all_boxes_affected_by_move(player_location, offset, game_map) {
                Ok(affected_boxes) => {
                    for (loc, _) in affected_boxes.iter() {
                        game_map.remove(loc);
                    }
                    for (loc, tile) in affected_boxes {
                        game_map.insert(loc + offset, tile);
                    }
                    Ok(PlayerPushedBoxes(new_location))
                }
                Err(_) => Ok(UnableToMove(MoveProblem::NoSpaceToPushBoxes)),
            }
        }
    }
}

fn find_all_boxes_affected_by_move(
    initial_position: IVec2,
    offset: IVec2,
    game_map: &HashMap<IVec2, SingleWidthTile>,
) -> miette::Result<HashMap<IVec2, SingleWidthTile>> {
    let mut done_map = HashMap::new();
    let mut open_list = VecDeque::from([initial_position + offset]);

    while let Some(new_pos) = open_list.pop_front() {
        if done_map.contains_key(&new_pos) {
            continue;
        }

        let maybe_tile_at_new_pos = game_map.get(&new_pos);
        debug!("evaluating {new_pos}: {maybe_tile_at_new_pos:?}");

        match maybe_tile_at_new_pos {
            None => {}
            Some(tile) => match tile {
                SingleWidthTile::Wall => {
                    return miette::bail!("Can't move - {tile:?} blocks location {new_pos}")
                }
                BoxOpen => {
                    let pos_other_box_tile = new_pos + IVec2::X;

                    done_map.insert(new_pos, BoxOpen);
                    //ok_list.insert(pos_other_box_tile, BoxClose);

                    open_list.push_back(new_pos + offset);
                    open_list.push_back(pos_other_box_tile);
                }
                BoxClose => {
                    let pos_other_box_tile = new_pos - IVec2::X;

                    //ok_list.insert(pos_other_box_tile, BoxOpen);
                    done_map.insert(new_pos, BoxClose);

                    open_list.push_back(new_pos + offset);
                    open_list.push_back(pos_other_box_tile);
                }
            },
        }
    }

    Ok(done_map)
}

fn move_player(
    game_map: &mut HashMap<IVec2, SingleWidthTile>,
    direction: &Direction,
    map_dimensions: IVec2,
    player_location: IVec2,
) -> miette::Result<MoveResult> {
    match direction {
        Direction::North | Direction::South => {
            move_player_vertically(game_map, direction, map_dimensions, player_location)
        }
        Direction::West | Direction::East => {
            move_player_horizontally(game_map, direction, map_dimensions, player_location)
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

    let game_map_with_player: HashMap<IVec2, Either<Player, OriginalTile>> = game_map_chars
        .iter()
        .enumerate()
        .flat_map(|(y, row)| {
            row.iter().enumerate().map(move |(x, char)| {
                let tile = match *char {
                    '.' => Right(OriginalTile::Empty),
                    'O' => Right(OriginalTile::Box),
                    '#' => Right(OriginalTile::Wall),
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
    let game_map: HashMap<IVec2, SingleWidthTile> = game_map_with_player
        .into_iter()
        .map(|(IVec2 { x, y }, either_player_or_tile)| {
            (
                IVec2 { x: x * 2, y },
                match either_player_or_tile {
                    Left(Player) => OriginalTile::Empty,
                    Right(tile) => tile,
                },
            )
        })
        .flat_map(|(loc, tile)| match tile {
            OriginalTile::Empty => {
                vec![]
            }
            OriginalTile::Wall => {
                vec![
                    (loc, SingleWidthTile::Wall),
                    (loc + IVec2::X, SingleWidthTile::Wall),
                ]
            }
            OriginalTile::Box => {
                vec![(loc, BoxOpen), (loc + IVec2::X, BoxClose)]
            }
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

        assert_eq!(player_location, IVec2::new(8, 4));

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

        assert_eq!(player_location, IVec2::new(8, 4));

        let mut game_map = original_game_map.clone();
        let result = move_player(
            &mut game_map,
            &Direction::West,
            map_dimensions,
            player_location,
        )?;

        let player_location = match result {
            UnableToMove(_) => player_location,
            PlayerMovedToEmptySpot(new_loc) => new_loc,
            PlayerPushedBoxes(new_loc) => new_loc,
        };

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

        assert_eq!(player_location, IVec2::new(7, 4));

        assert_eq!(game_map.get(&(player_location)), None);

        dbg!(game_map
            .iter()
            .filter(|(loc, _)| loc.y == player_location.y)
            .sorted_by_key(|(loc, _)| loc.x)
            .collect_vec());

        assert_eq!(
            game_map.get(&(player_location + IVec2::NEG_X)),
            Some(BoxClose).as_ref()
        );

        Ok(())
    }

    #[test]
    fn test_larger_example_push_t_shape_up() -> miette::Result<()> {
        let input = LARGER_EXAMPLE_STR;

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

<v<
"#
        .trim();

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

        // we can't create this scenario from parsing the original map, so we perform the moves
        let player_location = perform_moves(
            &mut game_map,
            movement_sequence,
            map_dimensions,
            player_location,
        )?;

        let actual_render_starting_pos = render_map(&game_map, map_dimensions, player_location);

        let expected_render_starting_pos = r#"
####################
##....[]....[]..[]##
##............[]..##
##..[][]....[]..[]##
##...[].......[]..##
##[]##@...[]......##
##[]....[]....[]..##
##..[][]..[]..[][]##
##........[]......##
####################
        "#
        .trim();

        assert_eq!(actual_render_starting_pos, expected_render_starting_pos);

        // moving north should push three boxes
        let player_location = perform_moves(
            &mut game_map,
            vec![Direction::North],
            map_dimensions,
            player_location,
        )?;

        let actual_render_final_pos = render_map(&game_map, map_dimensions, player_location);

        let expected_render_final_pos = r#"
####################
##....[]....[]..[]##
##..[][]......[]..##
##...[].....[]..[]##
##....@.......[]..##
##[]##....[]......##
##[]....[]....[]..##
##..[][]..[]..[][]##
##........[]......##
####################
        "#
        .trim();

        assert_eq!(actual_render_final_pos, expected_render_final_pos);

        Ok(())
    }

    #[test]
    fn test_larger_example_move_north_empty_space() -> miette::Result<()> {
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
            &Direction::North,
            map_dimensions,
            player_location,
        )?;

        let new_player_location = match result {
            UnableToMove(_) => player_location,
            PlayerMovedToEmptySpot(new_loc) => new_loc,
            PlayerPushedBoxes(new_loc) => new_loc,
        };

        let actual_render = render_map(&game_map, map_dimensions, new_player_location);

        let expected_render = r#"
####################
##....[]....[]..[]##
##............[]..##
##..[][]@...[]..[]##
##....[]......[]..##
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
    fn test_larger_example_move_south_empty_space() -> miette::Result<()> {
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
            &Direction::South,
            map_dimensions,
            player_location,
        )?;

        let new_player_location = match result {
            UnableToMove(_) => player_location,
            PlayerMovedToEmptySpot(new_loc) => new_loc,
            PlayerPushedBoxes(new_loc) => new_loc,
        };

        let actual_render = render_map(&game_map, map_dimensions, new_player_location);

        let expected_render = r#"
####################
##....[]....[]..[]##
##............[]..##
##..[][]....[]..[]##
##....[]......[]..##
##[]##..@.[]......##
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
    fn test_larger_example_push_box_east() -> miette::Result<()> {
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
        let mut new_player_location = player_location;
        for _ in 0..6 {
            let result = move_player(
                &mut game_map,
                &Direction::East,
                map_dimensions,
                new_player_location,
            )?;

            new_player_location = match result {
                UnableToMove(_) => player_location,
                PlayerMovedToEmptySpot(new_loc) => new_loc,
                PlayerPushedBoxes(new_loc) => new_loc,
            };
        }

        let actual_render = render_map(&game_map, map_dimensions, new_player_location);

        let expected_render = r#"
####################
##....[]....[]..[]##
##............[]..##
##..[][]....[]..[]##
##....[]......@[].##
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
    fn test_larger_example_push_two_boxes_east() -> miette::Result<()> {
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

        // insert a 2nd box right before the first one
        let mut game_map = original_game_map.clone();
        game_map.insert(IVec2::new(12, 4), BoxOpen);
        game_map.insert(IVec2::new(13, 4), BoxClose);

        let actual_render_initial_state = render_map(&game_map, map_dimensions, player_location);

        let expected_render_initial = r#"
####################
##....[]....[]..[]##
##............[]..##
##..[][]....[]..[]##
##....[]@...[][]..##
##[]##....[]......##
##[]....[]....[]..##
##..[][]..[]..[][]##
##........[]......##
####################
        "#
        .trim();

        assert_eq!(actual_render_initial_state, expected_render_initial);

        let mut new_player_location = player_location;
        for _ in 0..4 {
            let result = move_player(
                &mut game_map,
                &Direction::East,
                map_dimensions,
                new_player_location,
            )?;

            new_player_location = match result {
                UnableToMove(_) => new_player_location,
                PlayerMovedToEmptySpot(new_loc) => new_loc,
                PlayerPushedBoxes(new_loc) => new_loc,
            };
        }

        let actual_render = render_map(&game_map, map_dimensions, new_player_location);

        let expected_render = r#"
####################
##....[]....[]..[]##
##............[]..##
##..[][]....[]..[]##
##....[]....@[][].##
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
    fn test_larger_example_push_two_boxes_east_until_hit_wall() -> miette::Result<()> {
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

        // insert a 2nd box right before the first one
        let mut game_map = original_game_map.clone();
        game_map.insert(IVec2::new(12, 4), BoxOpen);
        game_map.insert(IVec2::new(13, 4), BoxClose);

        let actual_render_initial_state = render_map(&game_map, map_dimensions, player_location);

        let expected_render_initial = r#"
####################
##....[]....[]..[]##
##............[]..##
##..[][]....[]..[]##
##....[]@...[][]..##
##[]##....[]......##
##[]....[]....[]..##
##..[][]..[]..[][]##
##........[]......##
####################
        "#
        .trim();

        assert_eq!(actual_render_initial_state, expected_render_initial);

        let mut new_player_location = player_location;
        for _ in 0..7 {
            let result = move_player(
                &mut game_map,
                &Direction::East,
                map_dimensions,
                new_player_location,
            )?;

            new_player_location = match result {
                UnableToMove(_) => new_player_location,
                PlayerMovedToEmptySpot(new_loc) => new_loc,
                PlayerPushedBoxes(new_loc) => new_loc,
            };
        }

        let actual_render = render_map(&game_map, map_dimensions, new_player_location);

        let expected_render = r#"
####################
##....[]....[]..[]##
##............[]..##
##..[][]....[]..[]##
##....[].....@[][]##
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
