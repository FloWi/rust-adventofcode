use day_24::*;

fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[divan::bench]
fn part1() {
    part1::process(divan::black_box(include_str!("../input.txt",))).unwrap();
}

#[divan::bench(sample_count = 100)]
fn part2() {
    part2::process(divan::black_box(include_str!("../input.txt",))).unwrap();
}
