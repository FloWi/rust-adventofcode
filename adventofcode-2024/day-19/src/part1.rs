use crate::parser;
use itertools::Itertools;
use miette::miette;

#[tracing::instrument]
pub fn process(_input: &str) -> miette::Result<String> {
    let (_, problem_setup) = parser(_input).map_err(|e| miette!("parse failed {}", e))?;

    //dbg!(&problem_setup);

    let sorted_tokes = problem_setup
        .tokens
        .iter()
        .sorted_by_key(|token| token.len())
        .rev()
        .cloned()
        .collect_vec();

    let result = &problem_setup
        .towels
        .iter()
        .filter(|towel| match_towel(towel, &sorted_tokes))
        .count();

    Ok(result.to_string())
}

#[tracing::instrument]
fn match_towel(towel: &str, tokens: &Vec<&str>) -> bool {
    match_towel_recurse(towel, tokens)
}

#[tracing::instrument]
fn match_towel_recurse(towel: &str, tokens: &Vec<&str>) -> bool {
    if towel.is_empty() {
        return true;
    }
    let matching_tokens = tokens
        .iter()
        .filter(|token| towel.starts_with(**token))
        .collect_vec();

    if matching_tokens.is_empty() {
        return false;
    }

    for token in matching_tokens {
        let sub_string = &towel[token.len()..];
        if match_towel_recurse(sub_string, tokens) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
r, wr, b, g, bwu, rb, gb, br

brwrr
bggr
gbbr
rrbgbr
ubwu
bwurrg
brgr
bbrgwb
        "#
        .trim();
        assert_eq!("6", process(input)?);
        Ok(())
    }
}
