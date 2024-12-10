use itertools::Itertools;
use miette::miette;
use pathfinding::prelude::*;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let grid = create_grid(input)?;

    let can_move = |from: (usize, usize), to: (usize, usize)| -> bool {
        let from_value = grid[(from.0, from.1)];
        let to_value = grid[(to.0, to.1)];
        from_value
            .zip(to_value)
            .map(|(from, to)| to == from + 1)
            .unwrap_or(false)
    };

    // successors only return the neighbor node, not the cost of 1
    let successors = |pos: &(usize, usize)| {
        grid.neighbours(*pos, false)
            .filter(|&next_pos| can_move(*pos, next_pos))
            .collect::<Vec<_>>()
    };

    let starting_points: Vec<(usize, usize)> = grid
        .items()
        .filter_map(|(pos, value)| match value {
            Some(v) if *v == 0 => Some(pos),
            _ => None,
        })
        .collect_vec();

    let destination_points: Vec<(usize, usize)> = grid
        .items()
        .filter_map(|(pos, value)| match value {
            Some(v) if *v == 9 => Some(pos),
            _ => None,
        })
        .collect_vec();

    let result: usize = starting_points
        .into_iter()
        .flat_map(|start| destination_points.iter().map(move |dest| (start, dest)))
        .map(|(start, dest)| count_paths(start, successors, |&p| p == *dest))
        .sum();

    //dbg!(grid);

    Ok(result.to_string())
}

fn create_grid(input: &str) -> miette::Result<Matrix<Option<u32>>> {
    let matrix = Matrix::from_rows(
        input
            .lines()
            .map(|l| l.chars().map(|char| char.to_digit(10))),
    )
    .map_err(|e| miette!("parse failed {}", e))?;

    Ok(matrix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_1() -> miette::Result<()> {
        let input = r#"
.....0.
..4321.
..5..2.
..6543.
..7..4.
..8765.
..9....
        "#
        .trim();
        assert_eq!("3", process(input)?);
        Ok(())
    }

    #[test]
    fn test_example_2() -> miette::Result<()> {
        let input = r#"
..90..9
...1.98
...2..7
6543456
765.987
876....
987....
        "#
        .trim();
        assert_eq!("13", process(input)?);
        Ok(())
    }

    #[test]
    fn test_example_3() -> miette::Result<()> {
        let input = r#"
012345
123456
234567
345678
4.6789
56789.
        "#
        .trim();
        assert_eq!("227", process(input)?);
        Ok(())
    }

    #[test]
    fn test_example_4() -> miette::Result<()> {
        let input = r#"
89010123
78121874
87430965
96549874
45678903
32019012
01329801
10456732
        "#
        .trim();
        assert_eq!("81", process(input)?);
        Ok(())
    }
}
