use glam::IVec2;
use miette::miette;
use nom::bytes::complete::tag;
use nom::character::complete;
use nom::sequence::separated_pair;
use nom::IResult;
use std::ops::RangeInclusive;

pub fn process(input: &str) -> miette::Result<String> {
    process_parameterized(input, &(0..=70))
}

fn parse_args(args: &str) -> IResult<&str, RangeInclusive<i32>> {
    let (remaining, (range_from, range_to)) =
        separated_pair(complete::i32, tag("..="), complete::i32)(args)?;

    Ok((remaining, range_from..=range_to))
}

pub fn process_with_args(input: &str, args: &str) -> miette::Result<String> {
    let (_, min_savings_limit) = parse_args(args).map_err(|e| miette!("arg-parse failed {}", e))?;

    Ok(process_parameterized(input, &min_savings_limit)?)
}

#[tracing::instrument]
pub fn process_parameterized(
    input: &str,
    grid_limit: &RangeInclusive<i32>,
) -> miette::Result<String> {
    let (_, byte_locations): (&str, Vec<IVec2>) =
        crate::parse(input).map_err(|e| miette!("parse failed {}", e))?;

    let goal = IVec2::new(*grid_limit.end(), *grid_limit.end());

    let mut low = 0;
    let mut high = byte_locations.len();

    while low < high {
        let mid = (low + high) / 2;
        if crate::find_path(&byte_locations, &goal, mid + 1, grid_limit).is_some() {
            low = mid + 1;
        } else {
            high = mid;
        }
    }

    let result = byte_locations[low];

    Ok(format!("{},{}", result.x, result.y).to_string())
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
        assert_eq!("6,1", process_parameterized(input, &(0..=6))?);
        Ok(())
    }
}
