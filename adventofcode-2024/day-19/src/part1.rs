use itertools::Itertools;
use miette::miette;
use nom::bytes::complete::tag;
use nom::character::complete::*;
use nom::multi::separated_list1;
use nom::sequence::separated_pair;
use nom::IResult;

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
        .filter(|towel| match_towel_recurse(towel, &sorted_tokes))
        .count();

    Ok(result.to_string())
}

#[derive(Debug)]
struct ProblemSetup<'a> {
    tokens: Vec<&'a str>,
    towels: Vec<&'a str>,
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

fn parser(input: &str) -> IResult<&str, ProblemSetup> {
    let (rest, (tokens, towels)) = separated_pair(
        separated_list1(tag(", "), alpha1),
        multispace1,
        separated_list1(multispace1, alpha1),
    )(input.trim())?;

    Ok((rest, ProblemSetup { tokens, towels }))
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
