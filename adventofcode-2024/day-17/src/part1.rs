use crate::parse;
use itertools::Itertools;
use miette::miette;
use num_enum::TryFromPrimitive;
use std::ops::BitXor;

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, mut computer) = parse(input.trim()).map_err(|e| miette!("parse failed {}", e))?;

    println!("Input: \n{input}\n\nInitial_state: \n{computer:?}");

    computer.run();
    let output = computer
        .output
        .into_iter()
        .map(|num| num.to_string())
        .collect_vec()
        .join(",");

    println!("Output: {}", output);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
Register A: 729
Register B: 0
Register C: 0

Program: 0,1,5,4,3,0
        "#
        .trim();
        assert_eq!("4,6,3,5,6,3,5,2,1,0", process(input)?);
        Ok(())
    }

    #[test]
    fn test_example_1() -> miette::Result<()> {
        // If register C contains 9, the program 2,6 would set register B to 1.
        let input = r#"
Register A: 0
Register B: 0
Register C: 9

Program: 2,6
        "#
        .trim();

        let mut computer = parse(input).unwrap().1;
        computer.run();
        assert_eq!(computer.register_b, 1);
        Ok(())
    }
    #[test]
    fn test_example_2() -> miette::Result<()> {
        // If register A contains 10, the program 5,0,5,1,5,4 would output 0,1,2.
        let input = r#"
Register A: 10
Register B: 0
Register C: 0

Program: 5,0,5,1,5,4
        "#
        .trim();

        let mut computer = parse(input).unwrap().1;
        computer.run();

        assert_eq!(computer.output, vec![0, 1, 2]);
        Ok(())
    }

    #[test]
    fn test_example_3() -> miette::Result<()> {
        // If register A contains 2024, the program 0,1,5,4,3,0 would output 4,2,5,6,7,7,7,7,3,1,0 and leave 0 in register A.
        let input = r#"
Register A: 2024
Register B: 0
Register C: 0

Program: 0,1,5,4,3,0
        "#
        .trim();

        let mut computer = parse(input).unwrap().1;
        computer.run();

        assert_eq!(computer.output, vec![4, 2, 5, 6, 7, 7, 7, 7, 3, 1, 0]);
        assert_eq!(computer.register_a, 0);
        Ok(())
    }

    #[test]
    fn test_example_4() -> miette::Result<()> {
        // If register B contains 29, the program 1,7 would set register B to 26.
        let input = r#"
Register A: 0
Register B: 29
Register C: 0

Program: 1,7
        "#
        .trim();

        let mut computer = parse(input).unwrap().1;
        computer.run();

        assert_eq!(computer.register_b, 26);
        Ok(())
    }

    #[test]
    fn test_example_5() -> miette::Result<()> {
        // If register B contains 2024 and register C contains 43690, the program 4,0 would set register B to 44354.
        let input = r#"
Register A: 0
Register B: 2024
Register C: 43690

Program: 4,0
        "#
        .trim();

        let mut computer = parse(input).unwrap().1;
        computer.run();

        assert_eq!(computer.register_b, 44354);
        Ok(())
    }
}
