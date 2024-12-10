use itertools::Itertools;
use miette::miette;
use pathfinding::prelude::*;
use tracing::debug;

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
        // Can only move to positions with height difference of 1
    };

    let successors = |pos: &(usize, usize)| {
        grid.neighbours(*pos, false)
            .filter(|&next_pos| can_move(*pos, next_pos))
            .map(|next_pos| (next_pos, 1)) // Cost of 1 for each move
            .collect::<Vec<_>>()
    };

    let starting_points: Vec<(usize, usize)> = grid
        .items()
        .filter_map(|(pos, value)| match value {
            Some(v) if *v == 0 => Some(pos),
            _ => None,
        })
        .collect_vec();

    let result: usize = starting_points
        .into_iter()
        .map(|start| {
            let dijsktra_result = dijkstra_all(&start, successors)
                .into_iter()
                .filter(|(final_pos, _)| grid.at(final_pos.0, final_pos.1) == Some(9))
                .map(|(final_pos, _)| final_pos)
                .unique()
                .count();
            debug!("Starting from trailhead at {start:?} there are {dijsktra_result} valid paths");
            dijsktra_result
        })
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
0123
1234
8765
9876
        "#
        .trim();
        assert_eq!("1", process(input)?);
        Ok(())
    }

    #[test]
    fn test_example_2() -> miette::Result<()> {
        let input = r#"
...0...
...1...
...2...
6543456
7.....7
8.....8
9.....9
        "#
        .trim();
        assert_eq!("2", process(input)?);
        Ok(())
    }

    #[test]
    fn test_example_3() -> miette::Result<()> {
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
        assert_eq!("4", process(input)?);
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
        assert_eq!("36", process(input)?);
        Ok(())
    }
}
