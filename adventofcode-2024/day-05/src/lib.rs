use nom::{IResult, Parser};
use nom::multi::{many_m_n, separated_list1};
use nom::character::complete::{char, newline};
use nom::sequence::separated_pair;
use nom::character::complete;
use itertools::Itertools;

pub mod part1;
pub mod part2;

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

#[derive(Debug, Clone)]
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
