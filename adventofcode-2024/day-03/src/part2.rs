use itertools::Itertools;
use miette::miette;
use crate::MultiplyOperation;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, operations) = crate::all_mul_ops_parser(input)
        .map_err(|e| miette!("parse failed {}", e))?;

    // dbg!(&operations);

    let result: i32 = operations.iter().map(|MultiplyOperation(x, y )| x*y).sum();

    Ok(format!("{result}"))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))
        "#
            .trim();
        assert_eq!("161", process(input)?);
        Ok(())
    }
}
