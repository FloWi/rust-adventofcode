use crate::{has_correct_order, middle_number, parse, PageNumbersForUpdate, PageOrderingRule};
use itertools::Itertools;
use miette::miette;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_rest, (ordering_rules, pages_list)) =
        parse(input).map_err(|e| miette!("parse failed {}", e))?;

    dbg!(&ordering_rules);
    dbg!(&pages_list);

    let (valid_updates, invalid_updates): (Vec<_>, Vec<_>) = pages_list
        .iter()
        .partition(|pages| has_correct_order(*pages, ordering_rules.as_slice()));

    dbg!(&valid_updates);
    dbg!(&invalid_updates);

    let repaired_updates: Vec<_> = invalid_updates.iter().map(|pages| make_valid(*pages, ordering_rules.as_slice())).collect();

    dbg!(&repaired_updates);

    let result: i32 = repaired_updates
        .iter()
        .map(|PageNumbersForUpdate(pages)| middle_number(pages).unwrap())
        .sum();

    Ok(result.to_string())
}

fn make_valid(invalid_update: &PageNumbersForUpdate, rules: &[PageOrderingRule]) -> PageNumbersForUpdate {
    let original = invalid_update.0.clone();
    for permutation in original.clone().into_iter().permutations(invalid_update.0.len()) {
        let new_one = PageNumbersForUpdate(permutation.clone().into());
        if has_correct_order(&new_one, rules) {
            println!("make_valid: {:?} becomes {:?}", &original, permutation);
            return new_one;
        }
    }

    panic!("Should not happen - pinky promise")
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
