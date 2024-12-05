use itertools::Itertools;
use miette::miette;
use nom::Parser;
use crate::{parse, PageNumbersForUpdate};

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_rest, (ordering_rules, pages_list)) =
        parse(input).map_err(|e| miette!("parse failed {}", e))?;

    dbg!(&ordering_rules);
    dbg!(&pages_list);

    let valid_updates = pages_list
        .iter()
        .filter(|pages| crate::has_correct_order(*pages, ordering_rules.as_slice()))
        .collect_vec();
    dbg!(&valid_updates);

    let result: i32 = valid_updates
        .iter()
        .map(|PageNumbersForUpdate(pages)| crate::middle_number(pages).unwrap())
        .sum();

    Ok(result.to_string())
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
