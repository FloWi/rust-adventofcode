use crate::{Instruction, MultiplyOperation};
use itertools::Itertools;
use miette::miette;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, instructions) =
        crate::parse_instructions(input).map_err(|e| miette!("parse failed {}", e))?;

    // dbg!(&instructions);

    let mut enabled_operations = Vec::new();
    let mut is_enabled = true;
    for instruction in instructions {
        match instruction {
            Instruction::SetEnabled => is_enabled = true,
            Instruction::SetDisabled => is_enabled = false,
            Instruction::Work(operation) => {
                if is_enabled {
                    enabled_operations.push(operation)
                }
            }
        }
    }

    let result: i32 = enabled_operations
        .iter()
        .map(|MultiplyOperation(x, y)| x * y)
        .sum();

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
        assert_eq!("48", process(input)?);
        Ok(())
    }
}
