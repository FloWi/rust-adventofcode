use crate::parser;
use cached::Cached;
use itertools::Itertools;
use miette::miette;
use std::collections::HashMap;

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

    let mut cache: HashMap<&str, u64> = HashMap::new();

    let result = problem_setup
        .towels
        .iter()
        .filter_map(|towel| match_towel_recurse(towel, &sorted_tokes, &mut cache))
        .sum::<u64>();

    Ok(result.to_string())
}

fn match_towel_recurse<'a>(
    towel: &'a str,
    tokens: &Vec<&str>,
    cache: &mut HashMap<&'a str, u64>,
) -> Option<u64> {
    if towel.is_empty() {
        return Some(1);
    }
    if let Some(cached_entry) = cache.get(towel) {
        return Some(*cached_entry);
    }
    let matching_tokens = tokens
        .iter()
        .filter(|token| towel.starts_with(**token))
        .collect_vec();

    if matching_tokens.is_empty() {
        cache.insert(towel, 0);
        return None;
    }

    let result = matching_tokens
        .into_iter()
        .filter_map(|token| {
            let sub_string = &towel[token.len()..];
            match_towel_recurse(sub_string, tokens, cache)
        })
        .sum();

    cache.insert(towel, result);
    Some(result)
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
        assert_eq!("16", process(input)?);
        Ok(())
    }
}
