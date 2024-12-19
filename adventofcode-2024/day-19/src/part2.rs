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
        .filter(|towel| match_towel(towel, &sorted_tokes))
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
    if let Some(tokens) = match_towel_recurse(towel, tokens, Vec::new()) {
        println!("Found match for towel '{towel}'");
        println!("  matching tokens: '{tokens:?}'");
        true
    } else {
        false
    }
}

#[tracing::instrument]
fn match_towel_recurse(
    towel: &str,
    tokens: &Vec<&str>,
    tokens_used: Vec<String>,
) -> Option<Vec<String>> {
    if towel.is_empty() {
        return Some(tokens_used.clone());
    }
    let matching_tokens = tokens
        .iter()
        .filter(|token| towel.starts_with(**token))
        .collect_vec();

    if matching_tokens.is_empty() {
        return None;
    }

    for token in matching_tokens {
        let sub_string = &towel[token.len()..];
        let my_token_vec: Vec<String> = vec![(*token).to_string()];
        let new_tokens_used: Vec<String> = tokens_used
            .iter()
            .cloned()
            .chain(my_token_vec.into_iter())
            .collect_vec();
        if let Some(tokens_used) = match_towel_recurse(sub_string, tokens, new_tokens_used) {
            return Some(tokens_used);
        }
    }
    None
}

fn parser(input: &str) -> IResult<&str, ProblemSetup> {
    let (rest, (tokens, towels)) = separated_pair(
        separated_list1(tag(", "), alpha1),
        multispace1,
        separated_list1(multispace1, alpha1),
    )(input.trim())?;

    Ok((rest, ProblemSetup { tokens, towels }))
}

fn find_sub_tokens<'a>(token: &'a str, tokens: &'a [&'a str]) -> Vec<&'a str> {
    let possible_sub_tokens = tokens
        .iter()
        .filter(|sub| sub.len() < token.len())
        .filter(|&&sub| {
            let contains = token.contains(sub);
            let sub_tokens = token.split(sub).collect_vec();
            let all_sub_tokens_valid = sub_tokens
                .iter()
                .all(|sub| sub.is_empty() || tokens.contains(sub));
            dbg!(token, sub, contains, sub_tokens, all_sub_tokens_valid);
            contains && all_sub_tokens_valid
        })
        .cloned()
        .collect_vec();

    possible_sub_tokens.iter().unique().cloned().collect_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_unordered::assert_eq_unordered;

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
        assert_eq!("16", process(input)?);
        Ok(())
    }

    #[test]
    fn test_find_sub_tokens() {
        let tokens = vec!["r", "wr", "b", "g", "bwu", "rb", "gb", "br"];
        assert_eq_unordered!(find_sub_tokens("br", &tokens), vec!["r", "b"]);
    }

    #[test]
    fn test_find_sub_tokens_1() {
        let tokens = vec!["r", "b", "rb", "bg"];
        assert_eq_unordered!(find_sub_tokens("rbg", &tokens), vec!["r", "bg"]);
    }

    #[test]
    fn test_find_sub_tokens_broken_down() {
        let tokens = vec!["r", "b", "g", "rbgb", "rb", "gb"];
        assert_eq_unordered!(
            find_sub_tokens("rbgb", &tokens),
            vec!["r", "b", "g", "rb", "gb"]
        );
    }
}
