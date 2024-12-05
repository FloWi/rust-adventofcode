use itertools::Itertools;
use nom::character::complete;
use nom::character::complete::{char, newline};
use nom::multi::{many_m_n, separated_list1};
use nom::sequence::separated_pair;
use nom::{IResult, Parser};

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
    find_first_rule_that_breaks_update(pages, rules).is_none()
}

#[derive(Debug, Clone)]
struct UpdateBreaker {
    rule: PageOrderingRule,
    idx_1: usize,
    idx_2: usize,
}
fn find_first_rule_that_breaks_update(
    pages: &PageNumbersForUpdate,
    rules: &[PageOrderingRule],
) -> Option<UpdateBreaker> {
    rules
        .iter()
        .find_map(|rule @ PageOrderingRule(first, second)| {
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
                None => None,
                Some((idx_1, idx_2)) => {
                    if idx_1 > idx_2 {
                        Some(UpdateBreaker {
                            rule: (*rule).clone(),
                            idx_1,
                            idx_2,
                        })
                    } else {
                        None
                    }
                }
            }
        })
}

#[derive(Debug, Clone)]
struct PageOrderingRule(i32, i32);

#[derive(Debug, Clone)]
struct PageNumbersForUpdate(Vec<i32>);

impl PageNumbersForUpdate {
    pub(crate) fn swap_indices(&mut self, idx_1: usize, idx_2: usize) {
        self.0.swap(idx_1, idx_2)
    }
}

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
