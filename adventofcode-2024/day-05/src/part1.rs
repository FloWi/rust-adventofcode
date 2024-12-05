use itertools::Itertools;
use miette::miette;
use nom::character::complete;
use nom::character::complete::{char, newline};
use nom::multi::{many_m_n, separated_list1};
use nom::sequence::separated_pair;
use nom::IResult;
use nom::Parser;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_rest, (ordering_rules, pages_list)) =
        parse(input).map_err(|e| miette!("parse failed {}", e))?;

    dbg!(&ordering_rules);
    dbg!(&pages_list);

    let valid_updates = pages_list
        .iter()
        .filter(|pages| has_correct_order(*pages, ordering_rules.as_slice()))
        .collect_vec();
    dbg!(&valid_updates);

    let result: i32 = valid_updates
        .iter()
        .map(|PageNumbersForUpdate(pages)| middle_number(pages).unwrap())
        .sum();

    Ok(result.to_string())
}

fn middle_number(list: &Vec<i32>) -> Option<i32> {
    let idx = list.len() / 2;
    list.get(idx).cloned()
}

fn has_correct_order(pages: &PageNumbersForUpdate, rules: &[PageOrderingRule]) -> bool {
    /*
    (75,47,61,53,29)
    75 is correctly first because there are rules that put each other page after it: 75|47, 75|61, 75|53, and 75|29.
     */
    for PageOrderingRule(first, second) in rules {
        let maybe_first_idx = pages
            .0
            .iter()
            .find_position(|p| *p == first)
            .map(|(idx, _)| idx);
        let maybe_second_idx = pages
            .0
            .iter()
            .find_position(|p| *p == second)
            .map(|(idx, _)| idx);
        match maybe_first_idx.zip(maybe_second_idx) {
            None => {}
            Some((idx_1, idx_2)) => {
                if idx_1 > idx_2 {
                    return false;
                }
            }
        }
    }
    true
}

#[derive(Debug)]
struct PageOrderingRule(i32, i32);

#[derive(Debug)]
struct PageNumbersForUpdate(Vec<i32>);

fn parse(input: &str) -> IResult<&str, (Vec<PageOrderingRule>, Vec<PageNumbersForUpdate>)> {
    let ordering_rules_parser = separated_list1(
        newline,
        separated_pair(complete::i32, char('|'), complete::i32)
            .map(|(n1, n2)| PageOrderingRule(n1, n2)),
    );
    let pages_parser = separated_list1(
        newline,
        separated_list1(char(','), complete::i32).map(|pages| PageNumbersForUpdate(pages)),
    );

    let (remaining, result) =
        separated_pair(ordering_rules_parser, many_m_n(2, 2, newline), pages_parser)(input)?;

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
        assert_eq!("143", process(input)?);
        Ok(())
    }
}
