use day_18::*;

fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[divan::bench]
fn part1() {
    part1::process(
        divan::black_box(include_str!("../input.txt",)),
        &(0..=70),
        1024,
    )
    .unwrap();
}

#[divan::bench]
fn part2() {
    part2::process(divan::black_box(include_str!("../input.txt",)), &(0..=70)).unwrap();
}
