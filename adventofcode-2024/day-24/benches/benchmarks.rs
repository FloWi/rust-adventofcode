use day_24::*;

fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[divan::bench]
fn part1() {
    part1::process(divan::black_box(include_str!("../input1.txt",))).unwrap();
}

#[divan::bench(sample_count = 5)]
fn part2() {
    part2::process(divan::black_box(include_str!("../input2.txt",))).unwrap();
}
