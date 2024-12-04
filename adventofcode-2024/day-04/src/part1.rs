use itertools::Itertools;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let count = count_appearances( "XMAS" , input);

    Ok(count.to_string())
}

struct CharMatcher {
    char_to_match: char,
    x_offset: usize,
    y_offset: usize,
}

fn count_appearances(word: &str, input: &str) -> usize {
    let east = word.char_indices().map(|(o, char)| CharMatcher { char_to_match: char, x_offset: o, y_offset: 0 }).collect_vec();
    let west = word.char_indices().map(|(o, char)| CharMatcher { char_to_match: char, x_offset: word.len() - o, y_offset: 0 }).collect_vec();
    let south = word.char_indices().map(|(o, char)| CharMatcher { char_to_match: char, x_offset: 0, y_offset: o }).collect_vec();
    let north = word.char_indices().map(|(o, char)| CharMatcher { char_to_match: char, x_offset: 0, y_offset: word.len() - o }).collect_vec();

    let lines = input.lines().map(|line| line.chars().collect_vec()).collect_vec();

    let east_count = count_matches(&lines, &east);
    let west_count = count_matches(&lines, &west);
    let south_count = count_matches(&lines, &south);
    let north_count = count_matches(&lines, &north);


    dbg!(east_count);
    dbg!(west_count);
    dbg!(south_count);
    dbg!(north_count);

    east_count + west_count + south_count + north_count
}

fn count_matches(lines: &Vec<Vec<char>>, matcher: &Vec<CharMatcher>) -> usize {
    let height = lines.len();
    let width = lines.first().map(|line| line.len()).unwrap_or(0);

    let mut count = 0;

    for y in 0..height {
        for x in 0..width  {
            let found_match = matcher.iter().all(|cm| {
                let x = x + cm.x_offset;
                let y = y + cm.y_offset;
                if y > height - 1 || x > width - 1 {
                    false
                } else {
                    let char_to_test = lines[y][x];
                    cm.char_to_match == char_to_test
                }
            });

            if found_match  {
                count +=1;
            }
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
MMMSXXMASM
MSAMXMSMSA
AMXSXMAAMM
MSAMASMSMX
XMASAMXAMM
XXAMMXXAMA
SMSMSASXSS
SAXAMASAAA
MAMMMXMMMM
MXMXAXMASX
        "#
            .trim();

        // same as above, but irrelevant characters removed
        let input = r#"
....XXMAS.
.SAMXMS...
...S..A...
..A.A.MS.X
XMASAMX.MM
X.....XA.A
S.S.S.S.SS
.A.A.A.A.A
..M.M.M.MM
.X.X.XMASX
        "#
            .trim();
        assert_eq!("", process(input)?);
        Ok(())
    }
}
