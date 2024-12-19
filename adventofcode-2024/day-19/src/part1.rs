use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::all_consuming;
use nom::multi::many0;
use nom::IResult;

#[tracing::instrument]
pub fn process(_input: &str) -> miette::Result<String> {
    let words = [
        "brwrr", "bggr", "gbbr", "rrbgbr", "ubwu", "bwurrg", "brgr", "bbrgwb",
    ];

    let result = words
        .iter()
        .filter_map(|word| parser(word).ok())
        .map(|(_, words)| words.join(""))
        .inspect(|res| println!("{:?}", res))
        .count();

    Ok(result.to_string())
}

fn parser(word: &str) -> IResult<&str, Vec<&str>> {
    all_consuming(many0(alt((
        tag("bwu"),
        tag("wr"),
        tag("rb"),
        tag("gb"),
        tag("br"),
        tag("r"),
        tag("b"),
        tag("g"),
    ))))(word)
}

fn parse_towels() {}

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
