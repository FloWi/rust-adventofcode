use nom::IResult;

pub mod part1;
pub mod part2;

#[derive(Debug)]
struct ProblemSetup<'a> {
    tokens: Vec<&'a str>,
    towels: Vec<&'a str>,
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
