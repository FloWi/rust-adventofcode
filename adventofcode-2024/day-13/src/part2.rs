use itertools::Itertools;
use miette::miette;
use nom::AsChar;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, machines) = crate::parse(input).map_err(|e| miette!("parse failed {}", e))?;

    let result = machines
        .into_iter()
        .map(|machine| machine.with_fixed_amount_added_to_price_coords(10000000000000))
        .filter_map(crate::eval_machine)
        .map(|res| res.x * 3 + res.y * 1)
        .sum::<u64>();

    Ok(result.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
Button A: X+94, Y+34
Button B: X+22, Y+67
Prize: X=8400, Y=5400

Button A: X+26, Y+66
Button B: X+67, Y+21
Prize: X=12748, Y=12176

Button A: X+17, Y+86
Button B: X+84, Y+37
Prize: X=7870, Y=6450

Button A: X+69, Y+23
Button B: X+27, Y+71
Prize: X=18641, Y=10279
        "#
        .trim();
        assert_eq!("480", process(input)?);
        Ok(())
    }
}
