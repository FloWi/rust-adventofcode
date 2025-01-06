use day_18::part2::process_parameterized;
use miette::Context;

#[tracing::instrument]
fn main() -> miette::Result<()> {
    tracing_subscriber::fmt::init();

    let file = include_str!("../../input.txt");
    let result = process_parameterized(file, &(0..=70)).context("process part 2")?;
    println!("{}", result);
    Ok(())
}
