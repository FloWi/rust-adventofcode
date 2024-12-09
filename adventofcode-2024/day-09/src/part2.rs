use itertools::{repeat_n, Itertools};

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    // let mut disk_blocks = parse_disk_blocks(input.trim());
    // let compaction_result = compact_until_finished(&mut disk_blocks);
    // assert!(compaction_result);
    //
    // let result = compute_checksum(&disk_blocks);

    Ok("TODO".to_string())
}

#[derive(PartialOrd, PartialEq)]
enum CompactionResult {
    Finished,
    OneStepDone { id: u16 },
    Error,
    NoOp,
}

fn compact_one_chunk(disk_chunks: &mut Vec<Chunk>, max_id: Option<u16>) -> CompactionResult {
    println!("compact_one_chunk start");
    dbg!(&disk_chunks);
    if let Some(file_idx) = disk_chunks.iter().rposition(|chunk| match chunk {
        Chunk::File { id, .. } if *id <= max_id.unwrap_or(u16::MAX) => true,
        _ => false,
    }) {
        println!(
            "last valid file is at position: {file_idx}: {:?}",
            disk_chunks[file_idx]
        );
        let required_length = match disk_chunks[file_idx] {
            Chunk::File { length, .. } => length,
            _ => panic!("should be a file"),
        };

        if let Some(empty_idx) = disk_chunks.iter().position(|chunk| match chunk {
            Chunk::Empty { length, .. } if *length >= required_length => true,
            _ => false,
        }) {
            if empty_idx > file_idx {
                println!("empty_idx {empty_idx} is not < file_idx {file_idx}");
                CompactionResult::NoOp
            } else {
                println!(
                    "first matching empty block at position: {empty_idx}: {:?}",
                    disk_chunks[empty_idx]
                );
                match (disk_chunks[file_idx], disk_chunks[empty_idx]) {
                    (
                        Chunk::File {
                            id,
                            length: file_length,
                        },
                        Chunk::Empty {
                            length: empty_length,
                        },
                    ) => {
                        let empty_space_merge_candidate_idx = if file_length == empty_length {
                            println!("swapped equally sized (length {file_length} empty chunk with file chunk");
                            disk_chunks.swap(empty_idx, file_idx);
                            empty_idx
                        } else {
                            disk_chunks.swap(empty_idx, file_idx);
                            // the file was smaller than the empty space - insert a new empty space chunk after the file
                            let leftover_length = empty_length - file_length;
                            disk_chunks.insert(
                                empty_idx + 1,
                                Chunk::Empty {
                                    length: leftover_length,
                                },
                            );
                            // now the empty space (that has moved to the back) is too large. It needs to be trimmed to file_length.
                            println!("swapped file (length {file_length} with empty chunk (length {empty_length}) file chunk and inserted new empty chunk of length {leftover_length} after the file");
                            disk_chunks[file_idx + 1].set_length(file_length);
                            println!("set length of empty space at the end to {file_length}");
                            file_idx + 1
                        };
                        merge_empty_space(disk_chunks, empty_space_merge_candidate_idx);
                        CompactionResult::OneStepDone { id }
                    }
                    _ => panic!("should not happen"),
                }
            }
        } else {
            println!("no matching file found where id <= max_id({max_id:?})");
            CompactionResult::NoOp
        }
    } else {
        println!("no matching empty chunk found");
        CompactionResult::Finished
    }
}

fn merge_empty_space(disk_chunks: &mut Vec<Chunk>, empty_space_merge_candidate_idx: usize) {
    println!("checking if empty space at idx {empty_space_merge_candidate_idx} can be merged with surrounding empty space");
    dbg!(&disk_chunks);
    match (
        disk_chunks.get(empty_space_merge_candidate_idx - 1),
        disk_chunks.get(empty_space_merge_candidate_idx),
        disk_chunks.get(empty_space_merge_candidate_idx + 1),
    ) {
        (
            Some(Chunk::Empty { length: length_1 }),
            Some(Chunk::Empty { length: length_2 }),
            Some(Chunk::Empty { length: length_3 }),
        ) => {
            let total_length = length_1 + length_2 + length_3;
            println!("found 3 empty chunks of lengths ({length_1}, {length_2}, {length_3}) that can be merged to one chunk of length {total_length}");
            disk_chunks[empty_space_merge_candidate_idx - 1] = Chunk::Empty {
                length: total_length,
            };
            disk_chunks.remove(empty_space_merge_candidate_idx + 1);
            disk_chunks.remove(empty_space_merge_candidate_idx);
        }
        (Some(Chunk::Empty { length: length_1 }), Some(Chunk::Empty { length: length_2 }), _) => {
            let total_length = length_1 + length_2;
            println!("found 2 empty chunks of lengths ({length_1}, {length_2}) that can be merged to one chunk of length {total_length}");
            disk_chunks[empty_space_merge_candidate_idx - 1] = Chunk::Empty {
                length: total_length,
            };
            disk_chunks.remove(empty_space_merge_candidate_idx);
        }
        (_, Some(Chunk::Empty { length: length_1 }), Some(Chunk::Empty { length: length_2 })) => {
            let total_length = length_1 + length_2;
            println!("found 2 empty chunks of lengths ({length_1}, {length_2}) that can be merged to one chunk of length {total_length}");
            disk_chunks[empty_space_merge_candidate_idx] = Chunk::Empty {
                length: total_length,
            };
            disk_chunks.remove(empty_space_merge_candidate_idx + 1);
        }
        surroundings => {
            println!("no consecutive empty chunks found {surroundings:?}");
        }
    };
    dbg!(disk_chunks);
}

fn compute_checksum(disk_blocks: &Vec<Option<u16>>) -> usize {
    disk_blocks
        .iter()
        .enumerate()
        .fold(0usize, |acc, (idx, block_id)| {
            acc + idx * block_id.unwrap_or(0) as usize
        })
}

#[derive(Debug, Clone, Copy)]
enum Chunk {
    File { id: u16, length: u16 },
    Empty { length: u16 },
}

impl Chunk {
    pub(crate) fn set_length(&mut self, new_length: u16) {
        *self = match *self {
            Chunk::File { id, .. } => Chunk::File {
                id,
                length: new_length,
            },
            Chunk::Empty { .. } => Chunk::Empty { length: new_length },
        };
    }
}

fn parse_disk_chunks(input: &str) -> Vec<Chunk> {
    let disk_chunks = input
        .char_indices()
        .scan(0u16, |total, (idx, char)| {
            let idx = idx as u16;
            let length = match format!("{char}").parse::<u16>() {
                Ok(x) => x,
                Err(err) => {
                    panic!("error parsing char '{char}': {}", err)
                }
            };

            let chunk: Chunk = if idx % 2 != 0 {
                Chunk::Empty { length }
            } else {
                Chunk::File {
                    id: idx / 2,
                    length,
                }
            };

            *total += length;

            Some(chunk)
        })
        .filter(|chunk| match chunk {
            Chunk::File { length, .. } => *length > 0,
            Chunk::Empty { length, .. } => *length > 0,
        })
        .collect_vec();

    disk_chunks
}

fn render_disk_chunks(chunks: &[Chunk]) -> String {
    chunks
        .iter()
        .flat_map(|chunk| match chunk {
            Chunk::File { id, length } => repeat_n(id.to_string(), *length as usize),
            Chunk::Empty { length } => repeat_n(".".to_string(), *length as usize),
        })
        .join("")
}

/// Parses the disk blocks. -1 indicate an empty block. Might be premature optimization, but I think it's faster to use a number and not an enum / Option due to overhead
fn parse_disk_blocks(input: &str) -> Vec<Option<u16>> {
    let disk_blocks = input
        .char_indices()
        .flat_map(|(idx, char)| {
            let idx = idx as u16;
            let id: Option<u16> = if idx % 2 != 0 { None } else { Some(idx / 2) };
            let qty = match format!("{char}").parse::<i16>() {
                Ok(x) => x,
                Err(err) => {
                    panic!("error parsing char '{char}': {}", err)
                }
            };
            repeat_n(id, qty as usize)
        })
        .collect_vec();

    disk_blocks
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_parsing_disk_chunks() -> miette::Result<()> {
        let input = r#"
2333133121414131402
        "#
        .trim();
        let chunks = parse_disk_chunks(input);

        dbg!(&chunks);
        // assert_eq!(
        //     "00...111...2...333.44.5555.6666.777.888899",
        //     render_disk_blocks(&disk_blocks)
        // );
        Ok(())

        // example 1
        // input: 12345
        // disk layout of this input is:
        // 0..111....22222
        //
    }

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
2333133121414131402
        "#
        .trim();
        assert_eq!("2858", process(input)?);
        Ok(())

        // example 1
        // input: 12345
        // disk layout of this input is:
        // 0..111....22222
        //
    }

    #[test]
    fn test_complete_compaction() -> miette::Result<()> {
        let input = r#"
2333133121414131402
        "#
        .trim();

        unimplemented!();
        //
        // let mut disk_blocks = parse_disk_blocks(input);
        // let result = compact_until_finished(&mut disk_blocks);
        // assert!(result);
        //
        // assert_eq!(
        //     "00992111777.44.333....5555.6666.....8888..",
        //     render_disk_blocks(&disk_blocks)
        // );
        // assert_eq!(2858usize, compute_checksum(&disk_blocks));
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
        let mut disk_chunks = parse_disk_chunks(input);

        let expected = r#"
00...111...2...333.44.5555.6666.777.888899
0099.111...2...333.44.5555.6666.777.8888..
0099.1117772...333.44.5555.6666.....8888..
0099.111777244.333....5555.6666.....8888..
00992111777.44.333....5555.6666.....8888..
        "#
        .trim()
        .to_string();

        compact_one_chunk(&mut disk_chunks, None);
        assert_eq!(
            "0099.111...2...333.44.5555.6666.777.8888..",
            render_disk_chunks(&disk_chunks)
        );

        compact_one_chunk(&mut disk_chunks, Some(7));
        assert_eq!(
            "0099.1117772...333.44.5555.6666.....8888..",
            render_disk_chunks(&disk_chunks)
        );
        //can't move 6 - result unchanged
        compact_one_chunk(&mut disk_chunks, Some(6));
        assert_eq!(
            "0099.1117772...333.44.5555.6666.....8888..",
            render_disk_chunks(&disk_chunks)
        );
        //can't move 5 - result unchanged
        compact_one_chunk(&mut disk_chunks, Some(5));
        assert_eq!(
            "0099.1117772...333.44.5555.6666.....8888..",
            render_disk_chunks(&disk_chunks)
        );

        compact_one_chunk(&mut disk_chunks, Some(4));
        assert_eq!(
            "0099.111777244.333....5555.6666.....8888..",
            render_disk_chunks(&disk_chunks)
        );

        //can't move 3 - result unchanged
        compact_one_chunk(&mut disk_chunks, Some(3));
        assert_eq!(
            "0099.111777244.333....5555.6666.....8888..",
            render_disk_chunks(&disk_chunks)
        );

        compact_one_chunk(&mut disk_chunks, Some(2));
        assert_eq!(
            "00992111777.44.333....5555.6666.....8888..",
            render_disk_chunks(&disk_chunks)
        );

        //can't move 1 - result unchanged
        compact_one_chunk(&mut disk_chunks, Some(2));
        assert_eq!(
            "00992111777.44.333....5555.6666.....8888..",
            render_disk_chunks(&disk_chunks)
        );
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
