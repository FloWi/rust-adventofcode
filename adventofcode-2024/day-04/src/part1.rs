use itertools::Itertools;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let count = count_appearances( "XMAS" , input);

    Ok(count.to_string())
}

struct CharMatcher {
    char_to_match: char,
    x_offset: i32,
    y_offset: i32,
}

fn count_appearances(word: &str, input: &str) -> usize {

    let i32_char_indices: Vec<(i32, char)> =  word.char_indices().map(|(o, char)| (o as i32, char) ).collect_vec();

    let east = i32_char_indices.iter().map(|(o, char)| CharMatcher { char_to_match: *char, x_offset: *o, y_offset: 0 }).collect_vec();
    let west = i32_char_indices.iter().map(|(o, char)| CharMatcher { char_to_match: *char, x_offset: - *o, y_offset: 0 }).collect_vec();
    let south = i32_char_indices.iter().map(|(o, char)| CharMatcher { char_to_match: *char, x_offset: 0, y_offset: *o }).collect_vec();
    let north = i32_char_indices.iter().map(|(o, char)| CharMatcher { char_to_match: *char, x_offset: 0, y_offset: - o }).collect_vec();

    let north_east = i32_char_indices.iter().map(|(o, char)| CharMatcher { char_to_match: *char, x_offset: *o, y_offset: -o }).collect_vec();
    let south_east = i32_char_indices.iter().map(|(o, char)| CharMatcher { char_to_match: *char, x_offset: *o, y_offset: *o }).collect_vec();
    let south_west = i32_char_indices.iter().map(|(o, char)| CharMatcher { char_to_match: *char, x_offset: -o, y_offset: *o }).collect_vec();
    let north_west = i32_char_indices.iter().map(|(o, char)| CharMatcher { char_to_match: *char, x_offset: -o, y_offset: -o }).collect_vec();

    let lines = input.lines().map(|line| line.chars().collect_vec()).collect_vec();

    let east_count = count_matches(&lines, &east);
    let west_count = count_matches(&lines, &west);
    let south_count = count_matches(&lines, &south);
    let north_count = count_matches(&lines, &north);

    let north_east_count = count_matches(&lines, &north_east);
    let south_east_count = count_matches(&lines, &south_east);
    let south_west_count = count_matches(&lines, &south_west);
    let north_west_count = count_matches(&lines, &north_west);


    dbg!(east_count);
    dbg!(west_count);
    dbg!(south_count);
    dbg!(north_count);
    dbg!(north_east_count);
    dbg!(south_east_count);
    dbg!(south_west_count);
    dbg!(north_west_count);



    east_count + west_count + south_count + north_count + north_east_count + south_east_count + south_west_count + north_west_count
}

fn count_matches(lines: &Vec<Vec<char>>, matcher: &Vec<CharMatcher>) -> usize {
    let height = lines.len() as i32;
    let width = lines.first().map(|line| line.len()).unwrap_or(0) as i32;

    let mut count = 0;

    for y in 0..height {
        for x in 0..width  {
            let found_match = matcher.iter().all(|cm| {
                let x = x as i32 + cm.x_offset;
                let y = y as i32 + cm.y_offset;
                if y < 0 || x < 0 || y > height - 1 || x > width - 1 {
                    false
                } else {
                    let char_to_test = lines[y as usize][x as usize];
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
        assert_eq!("18", process(input)?);
        Ok(())
    }
}
