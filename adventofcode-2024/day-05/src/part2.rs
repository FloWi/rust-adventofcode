use crate::{
    find_first_rule_that_breaks_update, has_correct_order, middle_number, parse,
    PageNumbersForUpdate, PageOrderingRule, UpdateBreaker,
};
use itertools::Itertools;
use miette::miette;
use std::collections::HashSet;

//#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_rest, (ordering_rules, pages_list)) =
        parse(input).map_err(|e| miette!("parse failed {}", e))?;

    let invalid_updates = pages_list
        .iter()
        .filter(|pages| !has_correct_order(pages, ordering_rules.as_slice()))
        .collect_vec();

    let repaired_updates: Vec<_> = invalid_updates
        .iter()
        .map(|pages| make_valid(pages, ordering_rules.as_slice()))
        .collect();

    let result: i32 = repaired_updates
        .iter()
        .map(|PageNumbersForUpdate(pages)| middle_number(pages).unwrap())
        .sum();

    Ok(result.to_string())
}

fn make_valid(
    invalid_update: &PageNumbersForUpdate,
    rules: &[PageOrderingRule],
) -> PageNumbersForUpdate {
    /*
    make_valid: 10 out of 21 rules are relevant for invalid update PageNumbersForUpdate([97, 13, 75, 29, 47])
    PageOrderingRule(97, 13)
    PageOrderingRule(97, 47)
    PageOrderingRule(75, 29)
    PageOrderingRule(29, 13)
    PageOrderingRule(97, 29)
    PageOrderingRule(47, 13)
    PageOrderingRule(75, 47)
    PageOrderingRule(97, 75)
    PageOrderingRule(47, 29)
    PageOrderingRule(75, 13)
    update violated rule PageOrderingRule(29, 13). Affected indices: 3 and 1
    new version: PageNumbersForUpdate([97, 29, 75, 13, 47])
    update violated rule PageOrderingRule(75, 29). Affected indices: 2 and 1
    new version: PageNumbersForUpdate([97, 75, 29, 13, 47])
    update violated rule PageOrderingRule(47, 13). Affected indices: 4 and 3
    new version: PageNumbersForUpdate([97, 75, 29, 47, 13])
    update violated rule PageOrderingRule(47, 29). Affected indices: 3 and 2
    new version: PageNumbersForUpdate([97, 75, 47, 29, 13])
         */

    // find the rule that breaks the update
    // swap items
    // repeat

    let pages_set: HashSet<&i32> = HashSet::from_iter(invalid_update.0.iter());
    let relevant_rules: Vec<PageOrderingRule> = rules
        .iter()
        .filter(|PageOrderingRule(first, second)| {
            pages_set.contains(first) && pages_set.contains(second)
        })
        .cloned()
        .collect_vec();

    let mut wip_update = invalid_update.clone();

    while let Some(UpdateBreaker { idx_1, idx_2, .. }) =
        find_first_rule_that_breaks_update(&wip_update, &relevant_rules)
    {
        wip_update.swap_indices(idx_1, idx_2);
    }

    wip_update
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
        assert_eq!("123", process(input)?);
        Ok(())
    }
}
