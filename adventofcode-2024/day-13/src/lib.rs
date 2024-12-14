use glam::U64Vec2;
use nom::bytes::complete::take_till;
use nom::character::complete;
use nom::character::complete::line_ending;
use nom::multi::separated_list1;
use nom::sequence::{preceded, tuple};
use nom::IResult;

pub mod part1;
pub mod part2;

fn eval_machine(
    Machine {
        button_a,
        button_b,
        prize,
    }: Machine,
) -> Option<U64Vec2> {
    // took the solution from HyperNeutrino on YouTube
    let ax = button_a.x as f64;
    let ay = button_a.y as f64;
    let bx = button_b.x as f64;
    let by = button_b.y as f64;
    let px = prize.x as f64;
    let py = prize.y as f64;

    let ca = (px * by - py * bx) / (ax * by - ay * bx);
    let cb = (px - ax * ca) / bx;

    let ca_i64 = ca as u64;
    let cb_i64 = cb as u64;

    // only integer solutions are valid
    (ca == ca_i64 as f64 && cb == cb_i64 as f64).then_some(U64Vec2::new(ca_i64, cb_i64))
}

#[derive(Debug)]
struct Machine {
    button_a: U64Vec2,
    button_b: U64Vec2,
    prize: U64Vec2,
}

impl Machine {
    pub(crate) fn with_fixed_amount_added_to_price_coords(&self, fixed_amount: u64) -> Self {
        Self {
            prize: U64Vec2::new(self.prize.x + fixed_amount, self.prize.y + fixed_amount),
            ..*self
        }
    }
}

fn u_vec2_parser(input: &str) -> IResult<&str, U64Vec2> {
    // will match
    // Button B: X+22, Y+67
    // and
    // Prize: X=8400, Y=5400
    let (rest, (x, y)) = tuple((
        preceded(take_till(|c: char| c.is_numeric()), complete::u64),
        preceded(take_till(|c: char| c.is_numeric()), complete::u64),
    ))(input)?;

    Ok((rest, U64Vec2::new(x, y)))
}

fn machine_parser(input: &str) -> IResult<&str, Machine> {
    let (rest, (button_a, button_b, prize)) = tuple((
        u_vec2_parser,
        preceded(line_ending, u_vec2_parser),
        preceded(line_ending, u_vec2_parser),
    ))(input)?;

    Ok((
        rest,
        Machine {
            button_a,
            button_b,
            prize,
        },
    ))
}

fn parse(input: &str) -> IResult<&str, Vec<Machine>> {
    separated_list1(tuple((line_ending, line_ending)), machine_parser)(input)
}
