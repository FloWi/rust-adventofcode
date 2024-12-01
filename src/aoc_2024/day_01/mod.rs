use anyhow::Result;
use itertools::Itertools;

pub(crate) fn part1(input: &str) -> Result<String> {
    let numbers: Vec<(i32, i32)> = input.lines().map(|line| {
        let parts = line.split_whitespace().collect_vec();
        let part_0 = parts[0].parse::<i32>().unwrap();
        let part_1 = parts[1].parse::<i32>().unwrap();
        (part_0, part_1)
    }).collect();

    let (left, right): (Vec<_>, Vec<_>) = numbers.iter().cloned().unzip();

    let diffs = left.iter().sorted().zip(right.iter().sorted()).map(|(n1, n2)| {
        (n1, n2, (n1 - n2).abs())
    }).collect_vec();

    let total_distance: i32 = diffs.iter().map(|(_, _, diff)| diff).sum();

    println!("total_distance: {total_distance}");

    Ok(format!("{total_distance}"))
}

pub(crate) fn part2(input: &str) -> Result<String> {
    todo!()
}
