use itertools::Itertools;
use miette::miette;
use nom::character::complete::{alpha1, char, line_ending};
use nom::multi::separated_list1;
use nom::sequence::separated_pair;
use nom::IResult;
use petgraph::prelude::*;
use tracing::info;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, connections) = parse(input).map_err(|e| miette!("parse failed {}", e))?;

    let g = &UnGraphMap::<&str, ()>::from_edges(connections);

    if let Some(the_one) = find_the_largest_interconnected_cluster(g) {
        Ok(the_one)
    } else {
        panic!()
    }
}

fn find_the_largest_interconnected_cluster(g: &GraphMap<&str, (), Undirected>) -> Option<String> {
    let max_num_neighbors = g
        .nodes()
        .map(|node| g.neighbors(node).count())
        .max()
        .unwrap();
    info!("Max number of neighbors: {max_num_neighbors}");

    (0..max_num_neighbors).rev().find_map(|size| {
        info!("Trying to find interconnected cluster of size {size}");
        g.nodes().for_each(|node| {
            let neighbors = g.neighbors(node).collect_vec();
            info!(
                "node {node} has {} neighbors: {}",
                neighbors.len(),
                neighbors.iter().sorted().join(", ")
            )
        });

        let clusters = g
            .nodes()
            .flat_map(|node| {
                g.neighbors(node)
                    .combinations(size)
                    .filter_map(move |neighbor_candidates| {
                        let all_connected = neighbor_candidates
                            .iter()
                            .tuple_combinations()
                            .all(move |(a, b)| g.contains_edge(a, b));
                        if all_connected {
                            Some(
                                vec![node]
                                    .into_iter()
                                    .chain(neighbor_candidates)
                                    .sorted()
                                    .collect_vec(),
                            )
                        } else {
                            None
                        }
                    })
            })
            .unique()
            .take(2) //short circuit - we only need one, so we stop after finding a 2nd one.
            .collect_vec();

        match clusters.as_slice() {
            [the_one] => {
                info!("Found 1 interconnected cluster of size {size}");
                Some(the_one.join(","))
            }
            _ => {
                info!(
                    "Found 0 or more interconnected cluster of size {size}, but we only want one"
                );
                None
            }
        }
    })
}

fn parse(input: &str) -> IResult<&str, Vec<(&str, &str)>> {
    let (rest, connections) =
        separated_list1(line_ending, separated_pair(alpha1, char('-'), alpha1))(input)?;

    Ok((rest, connections.into_iter().collect_vec()))
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

        assert_eq!("co,de,ka,ta", process(input)?);
        Ok(())
    }
}
