use day_20::part2::process;
use miette::Context;

#[tracing::instrument]
fn main() -> miette::Result<()> {
    tracing_subscriber::fmt::init();

    let file = include_str!("../../input1.txt");
    let result = process(file, 100).context("process part 2")?;
    println!("{}", result);
    Ok(())
}
