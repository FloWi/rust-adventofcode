use crate::part1::Quadrant::{BottomLeft, BottomRight, TopLeft, TopRight};
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

    let median_width = width / 2;
    let median_height = height / 2;

    let (_, robots) = parse_robots(input).map_err(|e| miette!("parse failed {}", e))?;

    let seconds = 100;

    // dbg!(&robots);

    let quadrant_counts = robots
        .into_iter()
        .map(|robot| {
            let final_pos = robot.position + robot.velocity * seconds;
            IVec2::new(
                final_pos.x.rem_euclid(width),
                final_pos.y.rem_euclid(height),
            )
        })
        .filter_map(|pos| {
            let not_on_median = pos.x != median_width && pos.y != median_height;

            not_on_median.then_some(if pos.x > median_width {
                // RIGHT
                if pos.y > median_height {
                    BottomRight
                } else {
                    TopRight
                }
            } else {
                //LEFT
                if pos.y > median_height {
                    BottomLeft
                } else {
                    TopLeft
                }
            })
        })
        .counts();

    //dbg!(&quadrant_counts);

    let result = quadrant_counts.values().product::<usize>();

    Ok(result.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_GAME_FIELD_DIMENSIONS: IVec2 = IVec2::new(11, 7);

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
p=0,4 v=3,-3
p=6,3 v=-1,-3
p=10,3 v=-1,2
p=2,0 v=2,-1
p=0,0 v=1,3
p=3,0 v=-2,-2
p=7,6 v=-1,-3
p=3,0 v=-1,-2
p=9,3 v=2,3
p=7,3 v=-1,2
p=2,4 v=2,-3
p=9,5 v=-3,-3

        "#
        .trim();
        assert_eq!(
            "12",
            process_with_game_field_dimensions(input, TEST_GAME_FIELD_DIMENSIONS)?
        );
        Ok(())
    }
}
