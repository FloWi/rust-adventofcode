use miette::miette;
use nom::character::complete::{char, newline};
use nom::character::complete;
use nom::multi::{many_m_n, separated_list1};
use nom::sequence::separated_pair;
use nom::IResult;
use nom::Parser;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_rest, (ordering_rules, pages)) = parse(input)
        .map_err(|e| miette!("parse failed {}", e))?;

    dbg!(ordering_rules);
    dbg!(pages);
    Ok("???".to_string())
}

#[derive(Debug)]
struct PageOrderingRule(i32, i32);

#[derive(Debug)]
struct PageNumbersForUpdate(Vec<i32>);

fn parse(input: &str) -> IResult<&str, (Vec<PageOrderingRule>, Vec<PageNumbersForUpdate>)> {
    let ordering_rules_parser = separated_list1(newline, separated_pair(complete::i32, char('|'), complete::i32).map(|(n1, n2)| PageOrderingRule(n1, n2)));
    let pages_parser = separated_list1(newline, separated_list1(char(','), complete::i32).map(|pages| PageNumbersForUpdate(pages)));

    let (remaining, result) = separated_pair(ordering_rules_parser, many_m_n(2, 2, newline), pages_parser)(input)?;

    Ok((remaining, result))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
47|53
97|13
97|61
97|47
75|29
61|13
75|53
29|13
97|29
53|29
61|53
97|53
61|29
47|13
75|47
97|75
47|61
75|61
47|29
75|13
53|13

75,47,61,53,29
97,61,53,29,13
75,29,13
75,97,47,61,53
61,13,29
97,13,75,29,47
        "#
            .trim();
        assert_eq!("", process(input)?);
        Ok(())
    }
}
