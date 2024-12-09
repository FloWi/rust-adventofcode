use itertools::{repeat_n, Itertools};

#[tracing::instrument]
pub fn process(_input: &str) -> miette::Result<String> {
    //
    dbg!(i16::MAX);
    todo!("day 01 - part 1");
}

fn compact_files(disk_blocks: &mut Vec<Option<u16>>) {
    if let Some(first_empty_idx) = disk_blocks.iter().position(|b| b.is_none()) {
        if let Some(last_nonempty_idx) = disk_blocks.iter().rposition(|b| b.is_some()) {
            dbg!(first_empty_idx, last_nonempty_idx);
            disk_blocks.swap(first_empty_idx, last_nonempty_idx);
        }
    }
}

/// Parses the disk blocks. -1 indicate an empty block. Might be premature optimization, but I think it's faster to use a number and not an enum / Option due to overhead
fn parse_disk_blocks(input: &str) -> Vec<Option<u16>> {
    let disk_blocks = input
        .char_indices()
        .flat_map(|(idx, char)| {
            let idx = idx as u16;
            let id: Option<u16> = if idx % 2 != 0 { None } else { Some(idx / 2) };
            let qty = format!("{char}").parse::<i16>().unwrap();
            repeat_n(id, qty as usize)
        })
        .collect_vec();

    disk_blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
2333133121414131402
        "#
        .trim();
        assert_eq!("1928", process(input)?);
        Ok(())

        // example 1
        // input: 12345
        // disk layout of this input is:
        // 0..111....22222
        //
    }

    #[test]
    fn test_parsing() -> miette::Result<()> {
        let input = r#"
2333133121414131402
        "#
        .trim();
        let disk_blocks = parse_disk_blocks(input);
        dbg!(&disk_blocks);
        assert_eq!(
            "00...111...2...333.44.5555.6666.777.888899",
            render_disk_blocks(&disk_blocks)
        );
        Ok(())

        // example 1
        // input: 12345
        // disk layout of this input is:
        // 0..111....22222
        //
    }

    #[test]
    fn test_swapping_steps() -> miette::Result<()> {
        let input = r#"
2333133121414131402
        "#
        .trim();
        let mut disk_blocks = parse_disk_blocks(input);

        let expected = r#"
00...111...2...333.44.5555.6666.777.888899
009..111...2...333.44.5555.6666.777.88889.
0099.111...2...333.44.5555.6666.777.8888..
00998111...2...333.44.5555.6666.777.888...
009981118..2...333.44.5555.6666.777.88....
0099811188.2...333.44.5555.6666.777.8.....
009981118882...333.44.5555.6666.777.......
0099811188827..333.44.5555.6666.77........
00998111888277.333.44.5555.6666.7.........
009981118882777333.44.5555.6666...........
009981118882777333644.5555.666............
00998111888277733364465555.66.............
0099811188827773336446555566..............
        "#
        .trim()
        .to_string();

        let compacting_states: Vec<String> = (0..=12)
            .into_iter()
            .scan(disk_blocks, |current, idx| {
                if idx > 0 {
                    compact_files(current);
                }
                Some(render_disk_blocks(&current))
            })
            .collect_vec();

        let actual = compacting_states.join("\n");

        assert_eq!(actual, expected);
        Ok(())

        // example 1
        // input: 12345
        // disk layout of this input is:
        // 0..111....22222
        //
    }

    /// only makes sense to render disk blocks where max id < 10
    /// good enough for the small test examples
    fn render_disk_blocks(blocks: &Vec<Option<u16>>) -> String {
        blocks
            .iter()
            .map(|id| id.map(|id| id.to_string()).unwrap_or(".".to_string()))
            .join("")
    }
}
