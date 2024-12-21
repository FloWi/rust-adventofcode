use glam::IVec2;

pub mod part1;
pub mod part2;

/*
+---+---+---+
| 7 | 8 | 9 |
+---+---+---+
| 4 | 5 | 6 |
+---+---+---+
| 1 | 2 | 3 |
+---+---+---+
    | 0 | A |
    +---+---+
 */
const NUMERIC_KEY_MAP: [(IVec2, char); 11] = [
    (IVec2::new(0, 0), '7'),
    (IVec2::new(1, 0), '8'),
    (IVec2::new(2, 0), '9'),
    (IVec2::new(0, 1), '4'),
    (IVec2::new(1, 1), '5'),
    (IVec2::new(2, 1), '6'),
    (IVec2::new(0, 2), '1'),
    (IVec2::new(1, 2), '2'),
    (IVec2::new(2, 2), '3'),
    (IVec2::new(1, 3), '0'),
    (IVec2::new(2, 3), 'A'),
];

const DIRECTION_KEY_MAP: [(IVec2, char); 5] = [
    (IVec2::new(1, 0), '^'),
    (IVec2::new(2, 0), 'A'),
    (IVec2::new(0, 1), '<'),
    (IVec2::new(1, 1), 'v'),
    (IVec2::new(2, 1), '>'),
];
