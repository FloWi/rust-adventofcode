use glam::IVec2;
use miette::miette;
use std::ops::RangeInclusive;

#[tracing::instrument]
pub fn process(input: &str, grid_limit: &RangeInclusive<i32>) -> miette::Result<String> {
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
        assert_eq!("6,1", process(input, &(0..=6))?);
        Ok(())
    }
}
