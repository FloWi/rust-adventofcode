use itertools::Itertools;
use miette::miette;
use nom::character::complete::{alpha1, char, line_ending};
use nom::multi::separated_list1;
use nom::sequence::separated_pair;
use nom::IResult;
use std::collections::{HashMap, HashSet};

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, connections) = parse(input).map_err(|e| miette!("parse failed {}", e))?;

    let connection_map: HashMap<String, HashSet<String>> = connections
        .into_iter()
        .flat_map(|(from, to)| vec![(from.clone(), to.clone()), (to, from)])
        .into_grouping_map()
        .collect();

    let three_connected_computers = find_three_interconnected_computers(connection_map);

    let relevant_three_connected_computers = three_connected_computers
        .into_iter()
        .filter(|[comp_1, comp_2, comp_3]| {
            comp_1.starts_with("t") || comp_2.starts_with("t") || comp_3.starts_with("t")
        })
        .collect_vec();

    let result = relevant_three_connected_computers.len();

    Ok(result.to_string())
}

fn parse(input: &str) -> IResult<&str, Vec<(String, String)>> {
    let (rest, connections) =
        separated_list1(line_ending, separated_pair(alpha1, char('-'), alpha1))(input)?;

    Ok((
        rest,
        connections
            .into_iter()
            .map(|(comp_1, comp_2)| (comp_1.to_string(), comp_2.to_string()))
            .collect_vec(),
    ))
}

fn find_three_interconnected_computers(
    connection_map: HashMap<String, HashSet<String>>,
) -> HashSet<[String; 3]> {
    /*
    "aq": { "cg", "yn", "wq", "vc", },
    "yn": { "td", "wh", "cg", "aq", },
    "cg": { "aq", "de", "yn", "tb", },

    aq,cg,yn
    */
    let mut result = HashSet::new();
    for (comp_1, connections_1) in connection_map.iter() {
        for (comp_2, comp_3) in connections_1.iter().tuple_combinations() {
            let has_connections_between_2_3 = connection_map
                .get(comp_2)
                .map(|connections_2| connections_2.contains(comp_3))
                .unwrap_or(false);
            if has_connections_between_2_3 {
                let mut connections = [comp_1.clone(), comp_2.clone(), comp_3.clone()];
                connections.sort();
                result.insert(connections);
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
kh-tc
qp-kh
de-cg
ka-co
yn-aq
qp-ub
cg-tb
vc-aq
tb-ka
wh-tc
yn-cg
kh-ub
ta-co
de-co
tc-td
tb-wq
wh-td
ta-ka
td-qp
aq-cg
wq-ub
ub-vc
de-ta
wq-aq
wq-vc
wh-yn
ka-de
kh-ta
co-tc
wh-qp
tb-vc
td-yn
        "#
        .trim();

        /*
        In this example, there are 12 such sets of three inter-connected computers:

        aq,cg,yn
        aq,vc,wq
        co,de,ka
        co,de,ta
        co,ka,ta
        de,ka,ta
        kh,qp,ub
        qp,td,wh
        tb,vc,wq
        tc,td,wh
        td,wh,yn
        ub,vc,wq
                 */

        assert_eq!("7", process(input)?);
        Ok(())
    }
}
