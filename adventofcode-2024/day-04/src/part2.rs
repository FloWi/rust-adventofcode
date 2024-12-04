use itertools::Itertools;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let count = count_appearances(input);

    Ok(count.to_string())
}

#[derive(Debug)]
struct CharMatcher {
    char_to_match: char,
    x_offset: i32,
    y_offset: i32,
}

struct Direction {
    x_offset: i32,
    y_offset: i32,
}

impl Direction {
    fn reverse(&self) -> Direction {
        Direction {
            x_offset: -self.x_offset,
            y_offset: -self.y_offset,
        }
    }
}

const NE: Direction = Direction { x_offset: 1, y_offset: -1 };
const NW: Direction = Direction { x_offset: -1, y_offset: -1 };
const SE: Direction = Direction { x_offset: 1, y_offset: 1 };
const SW: Direction = Direction { x_offset: -1, y_offset: 1 };

fn count_appearances(input: &str) -> usize {
    /*
    this pattern has to be found
    A has to be in the middle
    M | S have to be on the outside

    M.S
    .A.
    M.S
     */

    let i32_char_indices: Vec<(i32, char)> = "MAS".char_indices().map(|(o, char)| (o as i32, char)).collect_vec();

    let north_east = create_matcher_by_direction(NE, &i32_char_indices);
    let south_east = create_matcher_by_direction(SE, &i32_char_indices);
    let south_west = create_matcher_by_direction(SW, &i32_char_indices);
    let north_west = create_matcher_by_direction(NW, &i32_char_indices);

    let lines = input.lines().map(|line| line.chars().collect_vec()).collect_vec();

    let ne_se_count = find_matches(&lines, &north_east, &south_east, "ne_se");
    let se_sw_count = find_matches(&lines, &south_east, &south_west, "se_sw");
    let nw_sw_count = find_matches(&lines, &north_west, &south_west, "nw_sw");
    let ne_nw_count = find_matches(&lines, &north_east, &north_west, "se_nw");

    ne_se_count + se_sw_count + nw_sw_count + ne_nw_count
}

fn create_matcher_by_direction(direction: Direction, i32_char_indices: &Vec<(i32, char)>) -> Vec<CharMatcher> {
    // everything needs to centered around the A (0,0).
    // If we are creating the SE matcher, we need to find the starting_point by going in the opposite direction (NW)

    let start = direction.reverse();
    let result = i32_char_indices.iter().map(|(o, char)| CharMatcher { char_to_match: *char, x_offset: start.x_offset + direction.x_offset * o, y_offset: start.y_offset + direction.y_offset * o }).collect_vec();
    result
}

fn check_both_matchers_at_location(lines: &Vec<Vec<char>>, matcher1: &Vec<CharMatcher>, matcher2: &Vec<CharMatcher>, width: &i32, height: &i32, x: i32, y: i32) -> bool {
    let found_match_1 = matcher1.iter().all(|cm| {
        check_match_for_char(cm, lines, x, y, width, height)
    });
    let found_match_2 = matcher2.iter().all(|cm| {
        check_match_for_char(cm, lines, x, y, width, height)
    });

    found_match_1 && found_match_2
}

fn check_match_for_char(cm: &CharMatcher, lines: &Vec<Vec<char>>, x: i32, y: i32, width: &i32, height: &i32) -> bool {
    let x = x + cm.x_offset;
    let y = y + cm.y_offset;
    
    if y < 0 || x < 0 || y > height - 1 || x > width - 1 {
        false
    } else {
        let char_to_test = lines[y as usize][x as usize];
        cm.char_to_match == char_to_test
    }
}
fn find_matches_for_matcher(lines: &Vec<Vec<char>>, matcher1: &Vec<CharMatcher>, matcher2: &Vec<CharMatcher>, width: &i32, height: &i32) -> Vec<(i32, i32)> {
    let mut matching_locations = Vec::new();

    for y in 0..*height {
        for x in 0..*width {
            let is_match = check_both_matchers_at_location(lines, matcher1, matcher2, width, height, x, y);

            if is_match {
                matching_locations.push((x, y));
            }
        }
    }
    matching_locations
}

fn find_matches(lines: &Vec<Vec<char>>, matcher1: &Vec<CharMatcher>, matcher2: &Vec<CharMatcher>, label: &str) -> usize {
    let height = lines.len() as i32;
    let width = lines.first().map(|line| line.len()).unwrap_or(0) as i32;

    let matches = find_matches_for_matcher(lines, matcher1, matcher2, &width, &height);

    matches.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specific_location() -> miette::Result<()> {
        let input = r#"
.M.S......
..A..MSMS.
.M.S.MAA..
..A.ASMSM.
.M.S.M....
..........
S.S.S.S.S.
.A.A.A.A..
M.M.M.M.M.
..........
        "#
            .trim();

        let i32_char_indices: Vec<(i32, char)> = "MAS".char_indices().map(|(o, char)| (o as i32, char)).collect_vec();


        let north_east = create_matcher_by_direction(NE, &i32_char_indices);
        let south_east = create_matcher_by_direction(SE, &i32_char_indices);
        let south_west = create_matcher_by_direction(SW, &i32_char_indices);
        let north_west = create_matcher_by_direction(NW, &i32_char_indices);

        dbg!(&north_east);
        dbg!(&south_east);
        dbg!(&south_west);
        dbg!(&north_west);

        let lines = input.lines().map(|line| line.chars().collect_vec()).collect_vec();
        let height = lines.len() as i32;
        let width = lines.first().map(|line| line.len()).unwrap_or(0) as i32;

        let actual = check_both_matchers_at_location(&lines, &north_east, &north_west, &width, &height, 7, 2);
        assert!(actual);
        Ok(())
    }

    #[test]
    fn test_process() -> miette::Result<()> {

        let input = r#"
.M.S......
..A..MSMS.
.M.S.MAA..
..A.ASMSM.
.M.S.M....
..........
S.S.S.S.S.
.A.A.A.A..
M.M.M.M.M.
..........
        "#
            .trim();
        assert_eq!("9", process(input)?);
        Ok(())
    }
}
