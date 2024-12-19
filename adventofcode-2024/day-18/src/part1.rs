use crate::find_path;
use glam::IVec2;
use miette::miette;
use nom::character::complete;
use nom::character::complete::char;
use nom::character::complete::line_ending;
use nom::combinator::map;
use nom::multi::separated_list1;
use nom::sequence::separated_pair;
use nom::IResult;
use std::ops::RangeInclusive;

#[tracing::instrument]
pub fn process(
    input: &str,
    grid_limit: &RangeInclusive<i32>,
    num_bytes: usize,
) -> miette::Result<String> {
    let (_, byte_locations) = parse(input).map_err(|e| miette!("parse failed {}", e))?;

    let goal = IVec2::new(*grid_limit.end(), *grid_limit.end());

    let Some((_path, cost)) = find_path(&byte_locations, &goal, num_bytes, grid_limit) else {
        panic!("No path found")
    };

    Ok(cost.to_string())
}

const NEIGHBORS: [IVec2; 4] = [IVec2::NEG_Y, IVec2::X, IVec2::Y, IVec2::NEG_X];

fn parse(input: &str) -> IResult<&str, Vec<IVec2>> {
    separated_list1(
        line_ending,
        map(
            separated_pair(complete::i32, char(','), complete::i32),
            |(x, y)| IVec2::new(x, y),
        ),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
5,4
4,2
4,5
3,0
2,1
6,3
2,4
1,5
0,6
3,3
2,6
5,1
1,2
5,5
2,5
6,5
1,4
0,4
6,4
1,1
6,1
1,0
0,5
1,6
2,0
        "#
        .trim();
        assert_eq!("22", process(input, &(0..=6), 12)?);
        Ok(())
    }
}
