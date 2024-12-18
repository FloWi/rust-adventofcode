use crate::{parse, Computer};
use itertools::Itertools;
use miette::miette;
use num_enum::TryFromPrimitive;
use std::ops::BitXor;

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, mut computer) = parse(input.trim()).map_err(|e| miette!("parse failed {}", e))?;

    println!("Input: \n{input}\n\nInitial_state: \n{computer:?}");

    let a = find_a(&mut computer);

    println!("a: {a}");
    Ok(a.to_string())
}

fn find_a(computer: &mut Computer) -> u64 {
    computer.run();

    42
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        // If register B contains 2024 and register C contains 43690, the program 4,0 would set register B to 44354.
        let input = r#"
Register A: 0
Register B: 0
Register C: 0

Program: 0,3,5,4,3,0
        "#
        .trim();


        assert_eq!(process(input)?, "117440");
        Ok(())
    }

    #[test]
    fn test_process_with_my_input() -> miette::Result<()> {
        // If register B contains 2024 and register C contains 43690, the program 4,0 would set register B to 44354.
        let input = r#"
Register A: 0
Register B: 0
Register C: 0

Program: 2,4,1,5,7,5,1,6,0,3,4,6,5,5,3,0
        "#
        .trim();


        let (_, computer) = parse(input.trim()).map_err(|e| miette!("parse failed {}", e))?;

        let mut results = vec![];

        let candidates = vec![
            17113115012403,
            17113115110707,
            17113115141427,
            17113115143475,
            17113115149619,
            17113115161907,
        ];

        for c in candidates {
            for v in 0..8 {
                let mut computer = computer.clone();
                let a = c * 8 + v;
                computer.register_a = a;
                computer.run();

                let out = computer.output;

                results.push((a, out));
            }
        }

        for (a, out) in results {
            println!("a: {a}, a_oct: {a:o}, a_binary: {a:b}, output: {out:?}");
        }

        assert_eq!(process(input)?, "136904920099226");
        Ok(())
    }
}
