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
use std::ops::Not;
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
                                "@"
                            } else {
                                match tile {
                                    Tile::Empty => ".",
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

impl Tile {
    fn width(&self) -> u32 {
        match self {
            Tile::Empty => 1,
            Tile::Wall => 2,
            Tile::Box => 2,
        }
    }
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

fn move_player_horizontally(
    game_map: &mut HashMap<IVec2, Tile>,
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
    let locations_affected_by_move = successors(Some(player_location.clone()), |pos| {
        let new_pos = pos + offset;
        let yield_result = x_range.contains(&new_pos.x).then_some(new_pos);
        yield_result
    })
    .skip(1)
    .collect_vec();

    let tiles_and_locations_affected_by_move = locations_affected_by_move
        .into_iter()
        .filter_map(|loc| game_map.get(&loc).cloned().map(|tile| (loc, tile.clone())))
        .collect_vec();

    dbg!(&tiles_and_locations_affected_by_move);

    match tiles_and_locations_affected_by_move.first() {
        None => miette::bail!("No move left (shouldn't happen)"),
        Some((_pos, Tile::Wall)) => Ok(UnableToMove(MoveProblem::PlayerDirectlyBlockedByWall)),
        Some((pos, Tile::Empty)) => Ok(PlayerMovedToEmptySpot(*pos)),
        Some((pos, Tile::Box)) => {
            // these are the scenarios
            // #.[]@    // push one box (move first box to first empty space)
            // #.[][]@  // push n boxes (move first box to first empty space)
            // #[]@..   // can't move, because no space between box and wall
            // #[][]@.  // can't move, because no space between boxes and wall

            //
            // #.[]@     // distance to first empty space: 3; distance to first wall: 4
            // #.[][]@   // distance to first empty space: 5; distance to first wall: 6
            // #[]@..    // distance to first empty space: None; distance to first wall: 2
            // #[][]@.   // distance to first empty space: None; distance to first wall: 2

            let boxes_affected_by_move = tiles_and_locations_affected_by_move
                .iter()
                .take_while(|(loc, tile)| matches!(tile, Tile::Box))
                .collect_vec();
            let first_tile_after_boxes = tiles_and_locations_affected_by_move
                .iter()
                .skip(boxes_affected_by_move.len())
                .next();

            dbg!(
                pos,
                &boxes_affected_by_move,
                first_tile_after_boxes,
                pos,
                player_location,
                direction
            );

            let row_before_move = game_map
                .clone()
                .into_iter()
                .filter(|(loc, _)| loc.y == player_location.y)
                .sorted_by_key(|(loc, _)| loc.x)
                .collect_vec();
            dbg!(row_before_move);

            match first_tile_after_boxes {
                None => {
                    miette::bail!("shouldn't happen. I expected the first tile after a move to be a wall or empty space")
                }
                Some((first_pos_after_boxes, tile)) => {
                    match tile {
                        Tile::Empty => {
                            game_map.remove(first_pos_after_boxes);
                            // All good, we can move all boxes one over (remove old entries and add new ones)
                            for (old_box_loc, _) in boxes_affected_by_move {
                                game_map.remove(&old_box_loc);
                                let new_box_loc = old_box_loc + offset;
                                game_map.insert(new_box_loc, Tile::Box);
                                println!("moved box from {old_box_loc} to {new_box_loc}");
                            }

                            // add an empty space at the players location
                            //game_map.insert(player_location, Tile::Empty);
                            game_map.insert(*pos, Tile::Empty);

                            let row_after_move = game_map
                                .clone()
                                .into_iter()
                                .filter(|(loc, _)| loc.y == player_location.y)
                                .sorted_by_key(|(loc, _)| loc.x)
                                .collect_vec();
                            dbg!(row_after_move);

                            Ok(PlayerPushedBoxes(*pos))
                        }
                        Tile::Wall => {
                            // We hit a wall - no-op
                            Ok(UnableToMove(MoveProblem::NoSpaceToPushBoxes))
                        }
                        Tile::Box => {
                            miette::bail!("shouldn't happen. I expected the first tile after a move to be a wall or empty space")
                        }
                    }
                }
            }
        }
    }
}

fn move_player_vertically(
    game_map: &mut HashMap<IVec2, Tile>,
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
    let maybe_tile_left_to_new_location = game_map.get(&(new_location + IVec2::new(-1, 0)));

    if matches!(maybe_tile_at_new_location, Some(Tile::Empty)) {
        Ok(PlayerMovedToEmptySpot(new_location))
    } else {
        dbg!(
            &new_location,
            &maybe_tile_at_new_location,
            &maybe_tile_left_to_new_location
        );

        todo!()
    }
}

fn move_player(
    game_map: &mut HashMap<IVec2, Tile>,
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
        .flat_map(|(loc, tile)| {
            match tile {
                Tile::Empty => {
                    // the empty tiles are also of width 2, but here we split them up into two tiles of width 1
                    vec![(loc, Tile::Empty), (loc + IVec2::X, Tile::Empty)]
                }
                tile => {
                    vec![(loc, tile)]
                }
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
        let mut new_player_location = player_location.clone();
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
        game_map.remove(&IVec2::new(12, 4));
        game_map.remove(&IVec2::new(13, 4));
        game_map.insert(IVec2::new(12, 4), Tile::Box);

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

        let mut new_player_location = player_location.clone();
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
        game_map.remove(&IVec2::new(12, 4));
        game_map.remove(&IVec2::new(13, 4));
        game_map.insert(IVec2::new(12, 4), Tile::Box);

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

        let mut new_player_location = player_location.clone();
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
