use day_18::part2::process;
use miette::Context;

#[tracing::instrument]
fn main() -> miette::Result<()> {
    tracing_subscriber::fmt::init();

    let file = include_str!("../../input1.txt");
    let result = process(file, &(0..=70)).context("process part 2")?;
    println!("{}", result);
    Ok(())
}