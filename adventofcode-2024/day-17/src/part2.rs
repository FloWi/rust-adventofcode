use crate::{parse, Computer};
use itertools::Itertools;
use miette::miette;
use num_enum::TryFromPrimitive;
use std::collections::HashSet;
use std::ops::BitXor;
use tracing::{debug, info};

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, mut computer) = parse(input.trim()).map_err(|e| miette!("parse failed {}", e))?;

    info!("Input: \n{input}\n\nInitial_state: \n{computer:?}");

    let a = find_a(computer);

    info!("a: {a}");
    Ok(a.to_string())
}

fn find_a(computer: Computer) -> u64 {
    let mut candidates: Vec<u64> = vec![0];
    let mut seen = HashSet::new();

    let mut c = computer.clone();
    while candidates.len() > 0 {
        let candidate = candidates.remove(0);

        for x in 0..8 {
            let a = candidate * 8 + x;
            if seen.contains(&a) {
                continue;
            } else {
                seen.insert(a);
            }
            debug!("find_a: a: {a}");

            c.reset();
            c.register_a = a;

            c.run();

            let output = c.output.clone();

            if output == computer.program {
                return a;
            }
            let num_digits = output.len();
            let last_elements = &c.program[&c.program.len() - num_digits..];

            if last_elements == output {
                candidates.push(a);
            }
        }
        candidates.sort();
    }

    panic!("Couldn't find solution")
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

        assert_eq!(process(input)?, "136904920099226");

        Ok(())
    }
}
