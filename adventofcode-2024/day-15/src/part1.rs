use glam::UVec2;
use itertools::Either::{Left, Right};
use itertools::{Either, Itertools};
use miette::miette;
use nom::character::complete::{line_ending, one_of};
use nom::multi::{many1, separated_list1};
use nom::sequence::{separated_pair, tuple};
use nom::IResult;
use std::cmp::PartialEq;
use std::collections::HashMap;

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

#[derive(Debug, PartialEq)]
enum Tile {
    Empty,
    Wall,
    Box,
}

#[derive(PartialEq)]
struct Player;

#[derive(Debug)]
struct Warehouse {
    game_map: HashMap<UVec2, Tile>,
    movement_sequence: Vec<Direction>,
    map_dimensions: UVec2,
    player_location: UVec2,
}

fn parse(input: &str) -> IResult<&str, Warehouse> {
    let (rest, (game_map_chars, moves)): (&str, (Vec<Vec<char>>, Vec<Vec<char>>)) = separated_pair(
        separated_list1(line_ending, many1(one_of("#.@O"))),
        tuple((line_ending, line_ending)),
        separated_list1(line_ending, many1(one_of("^>v<"))),
    )(input)?;

    let game_map_with_player: HashMap<UVec2, Either<Player, Tile>> = game_map_chars
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
                (UVec2::new(x as u32, y as u32), tile)
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
        .max_by_key(|UVec2 { x, y }| (x, y))
        .unwrap()
        + UVec2::new(1, 1);

    let player_location = *game_map_with_player
        .iter()
        .find(|(pos, tile)| **tile == Left(Player))
        .unwrap()
        .0;

    //create game_map with tiles only (player is removed)
    let game_map: HashMap<UVec2, Tile> = game_map_with_player
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
        assert_eq!("", process(input)?);
        Ok(())
    }
}
