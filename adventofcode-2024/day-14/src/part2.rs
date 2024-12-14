use glam::IVec2;
use itertools::Itertools;
use miette::miette;
use nom::bytes::complete::tag;
use nom::character::complete;
use nom::character::complete::{line_ending, space1};
use nom::multi::separated_list1;
use nom::sequence::{preceded, separated_pair};
use nom::IResult;
use nom::Parser;
use std::fs::File;
use std::io::Write;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    process_with_game_field_dimensions(input, IVec2::new(101, 103))
}
#[tracing::instrument]
pub fn process_with_game_field_dimensions(
    input: &str,
    game_field_dimensions: IVec2,
) -> miette::Result<String> {
    let width = game_field_dimensions.x;
    let height = game_field_dimensions.y;

    let (_, mut robots) = parse_robots(input).map_err(|e| miette!("parse failed {}", e))?;

    let mut seconds = 0;

    dbg!(&robots);

    let result = loop {
        seconds += 1;
        robots
            .iter_mut()
            .for_each(|r| r.position = (r.position + r.velocity).rem_euclid(game_field_dimensions));
        if robots.iter().map(|r| r.position).all_unique() {
            break seconds;
        }
    };

    let robots_txt = debug_robots(&robots, game_field_dimensions);

    let mut buffer = File::create("robots.txt").expect("unable to create file robots.txt");

    buffer
        .write_all(robots_txt.as_bytes())
        .expect("unable to write string to file");

    Ok(result.to_string())
}

fn debug_robots(robots: &[Robot], game_field_dimensions: IVec2) -> String {
    let position_counts = robots
        .iter()
        .map(|Robot { position, .. }| position)
        .counts();

    (0..game_field_dimensions.y)
        .map(|y| {
            let row: String = (0..game_field_dimensions.x)
                .map(|x| {
                    position_counts
                        .get(&IVec2::new(x, y))
                        .map(|num| num.to_string())
                        .unwrap_or(" ".to_string())
                })
                .join("");
            row
        })
        .join("\n")
}

#[derive(Debug, Eq, PartialEq, Hash)]
enum Quadrant {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

fn parse_i_vec2(input: &str) -> IResult<&str, IVec2> {
    let (remaining, (x, y)) = separated_pair(complete::i32, tag(","), complete::i32)(input)?;

    Ok((remaining, IVec2::new(x, y)))
}

#[derive(Debug)]
struct Robot {
    position: IVec2,
    velocity: IVec2,
}

fn parse_robots(input: &str) -> IResult<&str, Vec<Robot>> {
    let (rest, robots) = separated_list1(
        line_ending,
        separated_pair(
            preceded(tag("p="), parse_i_vec2),
            space1,
            preceded(tag("v="), parse_i_vec2),
        )
        .map(|(position, velocity)| Robot { position, velocity }),
    )(input)?;

    Ok((rest, robots))
}
