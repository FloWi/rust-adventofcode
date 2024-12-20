use cached::proc_macro::cached;
use cached::Cached;
use itertools::Itertools;
use miette::miette;
use nom::IResult;
use tracing::{debug, info};

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

    let result = problem_setup
        .towels
        .iter()
        .filter_map(|towel| match_towel_recurse(towel, &sorted_tokes))
        .sum::<u64>();

    {
        use cached::Cached;

        let mut cache = MATCH_TOWEL_RECURSE.lock().unwrap();

        info!("[cached] size {:?}", cache.cache_size());
        info!("[cached] hits {:?}", cache.cache_hits().unwrap_or(0));
        info!("[cached] misses {:?}", cache.cache_misses().unwrap_or(0));
        cache.cache_clear();
        info!("Cleared cache");

    }
    Ok(result.to_string())

}

#[derive(Debug)]
struct ProblemSetup<'a> {
    tokens: Vec<&'a str>,
    towels: Vec<&'a str>,
}
#[cached(key = "String", convert = r##"{ format!("{towel}") }"##)]
fn match_towel_recurse(towel: &str, tokens: &Vec<&str>) -> Option<u64> {
    if towel.is_empty() {
        return Some(1);
    }
    let matching_tokens = tokens
        .iter()
        .filter(|token| towel.starts_with(**token))
        .collect_vec();

    if matching_tokens.is_empty() {
        return None;
    }

    Some(
        matching_tokens
            .into_iter()
            .filter_map(|token| {
                let sub_string = &towel[token.len()..];
                match_towel_recurse(sub_string, tokens)
            })
            .sum(),
    )
}

fn parser(input: &str) -> IResult<&str, ProblemSetup> {
    use nom::bytes::complete::tag;
    use nom::character::complete::*;
    use nom::multi::separated_list1;
    use nom::sequence::separated_pair;

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
        assert_eq!("16", process(input)?);
        Ok(())
    }
}
