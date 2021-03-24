use std::ops::RangeInclusive;

use regex_syntax::hir::ClassUnicodeRange;

use crate::regex::stateid::StateId;
use crate::regex::NFA;

// After the first byte in utf, we expect bytes in this range.
const ONWARDS: RangeInclusive<u8> = 128u8..=191;

const FIRST: RangeInclusive<u8> = 0u8..=127;
const SECOND: RangeInclusive<u8> = 192u8..=223;
const THIRD: RangeInclusive<u8> = 224u8..=239;
const FOURTH: RangeInclusive<u8> = 240..=247;

/*
Tabele for utf-8 byte layout. We need to generate a nfa that recognises a range.
Its sort of like creating a regular expression for a number between n and m.
You think its easy, untill you realise there are a lot of edgecases.

+-----------+-----------+-----------+-----------+----------------------+-----------------------------------+
| 1st Byte  | 2nd Byte  | 3rd Byte  | 4th Byte  | Number of Free Bits  | Maximum Expressible Unicode Value |
+-----------+-----------+-----------+-----------+----------------------+-----------------------------------+
| 0xxxxxxx  |           |           |           | 7                    | 007F hex (127)                    |
| 110xxxxx  | 10xxxxxx  |           |           | (5+6)=11             | 07FF hex (2047)                   |
| 1110xxxx  | 10xxxxxx  | 10xxxxxx  |           | (4+6+6)=16           | FFFF hex (65535)                  |
| 11110xxx  | 10xxxxxx  | 10xxxxxx  | 10xxxxxx  | (3+6+6+6)=21         | 10FFFF hex (1,114,111)            |
+-----------+-----------+-----------+-----------+----------------------+-----------------------------------+


+-----------+-----------+-----------+-----------+----------------------+-----------------------------------+
| 1st Byte  | 2nd Byte  | 3rd Byte  | 4th Byte  | Number of Free Bits  | Maximum Expressible Unicode Value |
+-----------+-----------+-----------+-----------+----------------------+-----------------------------------+
|  0..=191  |           |           |           | 7                    | 007F hex (127)                    |
| 192..=223 | 128..=191 |           |           | (5+6)=11             | 07FF hex (2047)                   |
| 224..=239 | 128..=191 | 128..=191 |           | (4+6+6)=16           | FFFF hex (65535)                  |
| 240..=247 | 128..=191 | 128..=191 | 128..=191 | (3+6+6+6)=21         | 10FFFF hex (1,114,111)            |
+-----------+-----------+-----------+-----------+----------------------+-----------------------------------+

*/

// This is used for the 2 byte onwards
fn between2<A: std::hash::Hash + Eq + Copy + std::fmt::Debug>(
    nfa: &mut NFA<A>,
    from: [u8; 2],
    to: [u8; 2],
    root: StateId,
) {
    match (from[0] == to[0], from[1] == to[1]) {
        (true, true) => {
            let a = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_connection(root, a, from[0]);
            nfa.push_connection(a, e, from[1]);
            nfa.push_end(e)
        }
        (true, false) => {
            let a = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_connection(root, a, from[0]);
            nfa.push_connections(a, e, from[1]..=to[1]);
            nfa.push_end(e)
        }
        (false, true) | (false, false) => {
            // Match from[0]
            let a = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_connection(root, a, from[0]);
            nfa.push_connections(a, e, from[1]..192);
            nfa.push_end(e);

            // Something between from[0] and to[0]
            // @TODO check add  from[0] = 1, to[0] = 2
            if from[0] as usize + 1 < to[0] as usize {
                let a = nfa.push_state();
                let e = nfa.push_state();

                nfa.push_connections(root, a, (from[0] + 1)..to[0]);
                nfa.push_connections(a, e, ONWARDS);

                nfa.push_end(e);
            }

            // from[0] = 190, from[1] = 191
            // to[0] = 191   to[1] = 191
            // Match to[0]
            let a = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_connection(root, a, to[0]);
            nfa.push_connections(a, e, 128..=to[1]);
            nfa.push_end(e)
        }
    }
}

// This is used for the 2 byte onwards
fn between3<A: std::hash::Hash + Eq + Copy + std::fmt::Debug>(
    nfa: &mut NFA<A>,
    from: [u8; 3],
    to: [u8; 3],
    root: StateId,
) {
    match (from[0] == to[0], from[1] == to[1], from[2] == to[2]) {
        (true, true, true) => {
            let a = nfa.push_state();
            let b = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_end(e);

            nfa.push_connection(root, a, from[0]);
            nfa.push_connection(a, b, from[1]);
            nfa.push_connection(b, e, from[2]);
        }
        (true, true, false) => {
            let a = nfa.push_state();
            let b = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_end(e);

            nfa.push_connection(root, a, from[0]);
            nfa.push_connection(a, b, from[1]);
            nfa.push_connections(b, e, from[2]..=to[2]);
        }
        (true, false, _) => {
            let a = nfa.push_state();
            nfa.push_connection(root, a, from[0]);
            between2(nfa, [from[1], from[2]], [to[1], to[2]], a);
        }
        (false, _, _) => {
            // If equals to from[0]
            let a = nfa.push_state();
            nfa.push_connection(root, a, from[0]);
            // Now we just want a value bigger or equal to from[1..]
            between2(nfa, [from[1], from[2]], [191, 191], a);

            // If there is a value between from[0] and to[0]
            if from[0] as usize + 1 < to[0] as usize {
                let a = nfa.push_state();
                nfa.push_connections(root, a, from[0] + 1..to[0]);
                // Now we just want a arbitrary value in the valid byterange
                between2(nfa, [128, 128], [191, 191], a);
            }

            // If equals to the top
            let a = nfa.push_state();
            nfa.push_connection(root, a, to[0]);
            // Now we just want a value bigger or equal to from[1..]
            between2(nfa, [128, 128], [to[1], to[2]], a);
        }
    }
}
#[allow(clippy::many_single_char_names)]
fn any_char_of_length<A: std::hash::Hash + Eq + Copy + std::fmt::Debug>(
    nfa: &mut NFA<A>,
    n: usize,
) {
    match n {
        1 => {
            let e = nfa.push_state();
            nfa.push_connections(StateId::of(0), e, FIRST);
            nfa.push_end(e);
        }
        2 => {
            let a = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_connections(StateId::of(0), a, SECOND);
            nfa.push_connections(a, e, ONWARDS);
            nfa.push_end(e)
        }
        3 => {
            let a = nfa.push_state();
            let b = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_connections(StateId::of(0), a, THIRD);
            nfa.push_connections(a, b, ONWARDS);
            nfa.push_connections(b, e, ONWARDS);
            nfa.push_end(e)
        }
        4 => {
            let a = nfa.push_state();
            let b = nfa.push_state();
            let c = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_connections(StateId::of(0), a, FOURTH);
            nfa.push_connections(a, b, ONWARDS);
            nfa.push_connections(b, c, ONWARDS);
            nfa.push_connections(c, e, ONWARDS);
            nfa.push_end(e)
        }
        _ => {
            panic!("Not a valid length of a utf-8 char")
        }
    }
}

/// Modifies the nfa such that it recognises any char that has the length of from, but is less then or equal to it
fn below_or_eq_for_given_length<A: std::hash::Hash + Eq + Copy + std::fmt::Debug>(
    nfa: &mut NFA<A>,
    from: [u8; 4],
) {
    match from[0] {
        0..=191 => {
            // Length is one
            let e = nfa.push_state();
            nfa.push_connections(StateId::of(0), e, 0..=from[0]);
            nfa.push_end(e)
        }
        192..=223 => {
            // Length is dois

            // First value exact match
            let a = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_end(e);
            nfa.push_connection(StateId::of(0), a, from[0]);
            nfa.push_connections(a, e, 128..=from[1]);

            // First value less,
            let a = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_end(e);
            nfa.push_connections(StateId::of(0), a, 192..from[0]);
            nfa.push_connections(a, e, ONWARDS);
        }
        224..=239 => {
            // Length is treis

            // First less
            let a = nfa.push_state();
            let b = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_end(e);
            nfa.push_connections(StateId::of(0), a, 0..from[0]);
            nfa.push_connections(a, b, ONWARDS);
            nfa.push_connections(b, e, ONWARDS);

            // First value exact match, but second less
            let a = nfa.push_state();
            let b = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_end(e);
            nfa.push_connection(StateId::of(0), a, from[0]);
            nfa.push_connections(a, b, 128..from[1]);
            nfa.push_connections(b, e, ONWARDS);

            // First and second exact
            let a = nfa.push_state();
            let b = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_end(e);
            nfa.push_connection(StateId::of(0), a, from[0]);
            nfa.push_connection(a, b, from[1]);
            nfa.push_connections(b, e, 128..=from[2]);
        }
        240..=247 => {
            // Length quatro

            // First less
            let a = nfa.push_state();
            let b = nfa.push_state();
            let c = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_end(e);
            nfa.push_connections(StateId::of(0), a, 0..from[0]);
            nfa.push_connections(a, b, ONWARDS);
            nfa.push_connections(b, c, ONWARDS);
            nfa.push_connections(c, e, ONWARDS);

            // First eq, second less
            let a = nfa.push_state();
            let b = nfa.push_state();
            let c = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_end(e);
            nfa.push_connection(StateId::of(0), a, from[0]);
            nfa.push_connections(a, b, 128..from[1]);
            nfa.push_connections(b, c, ONWARDS);
            nfa.push_connections(c, e, ONWARDS);

            // First & second eq, third less
            let a = nfa.push_state();
            let b = nfa.push_state();
            let c = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_end(e);
            nfa.push_connection(StateId::of(0), a, from[0]);
            nfa.push_connection(a, b, from[1]);
            nfa.push_connections(b, c, 128..from[2]);
            nfa.push_connections(c, e, ONWARDS);

            // First & second & third eq, last less or eq
            let a = nfa.push_state();
            let b = nfa.push_state();
            let c = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_end(e);
            nfa.push_connection(StateId::of(0), a, from[0]);
            nfa.push_connection(a, b, from[1]);
            nfa.push_connection(b, c, from[2]);
            nfa.push_connections(c, e, 128..=from[3]);
        }

        _ => {
            panic!("not a valid start on suposed utf-8 bytes")
        }
    }
}

// Modifies the nfa such that it recognises any char that has the length of from, but is bigger or equal to it.
fn more_or_eq_for_given_length<A: std::hash::Hash + Eq + Copy + std::fmt::Debug>(
    nfa: &mut NFA<A>,
    from: [u8; 4],
) {
    match from[0] {
        0..=191 => {
            // Length is one
            let e = nfa.push_state();
            nfa.push_connections(StateId::of(0), e, from[0]..=191);
            nfa.push_end(e)
        }
        192..=223 => {
            // Length is dois

            // First value exact match
            let a = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_end(e);
            nfa.push_connection(StateId::of(0), a, from[0]);
            nfa.push_connections(a, e, from[1]..=191);

            // First value more,
            let a = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_end(e);
            nfa.push_connections(StateId::of(0), a, from[0] + 1..=223u8);
            nfa.push_connections(a, e, ONWARDS);
        }
        224..=239 => {
            // Length is treis

            // First more
            let a = nfa.push_state();
            let b = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_end(e);
            nfa.push_connections(StateId::of(0), a, from[0] + 1..=239);
            nfa.push_connections(a, b, ONWARDS);
            nfa.push_connections(b, e, ONWARDS);

            // First value exact match, but second more
            let a = nfa.push_state();
            let b = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_end(e);
            nfa.push_connection(StateId::of(0), a, from[0]);
            nfa.push_connections(a, b, from[1] + 1..=191);
            nfa.push_connections(b, e, ONWARDS);

            // First and second exact
            let a = nfa.push_state();
            let b = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_end(e);
            nfa.push_connection(StateId::of(0), a, from[0]);
            nfa.push_connection(a, b, from[1]);
            nfa.push_connections(b, e, from[2]..=191);
        }
        240..=247 => {
            // Length quatro

            // First more
            let a = nfa.push_state();
            let b = nfa.push_state();
            let c = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_end(e);
            nfa.push_connections(StateId::of(0), a, from[0] + 1..247);
            nfa.push_connections(a, b, ONWARDS);
            nfa.push_connections(b, c, ONWARDS);
            nfa.push_connections(c, e, ONWARDS);

            // First eq, second more
            let a = nfa.push_state();
            let b = nfa.push_state();
            let c = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_end(e);
            nfa.push_connection(StateId::of(0), a, from[0]);
            nfa.push_connections(a, b, from[1]..=191);
            nfa.push_connections(b, c, ONWARDS);
            nfa.push_connections(c, e, ONWARDS);

            // First & second eq, third more
            let a = nfa.push_state();
            let b = nfa.push_state();
            let c = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_end(e);
            nfa.push_connection(StateId::of(0), a, from[0]);
            nfa.push_connection(a, b, from[1]);
            nfa.push_connections(b, c, from[2] + 1..191);
            nfa.push_connections(c, e, ONWARDS);

            // First & second & third eq, last less or eq
            let a = nfa.push_state();
            let b = nfa.push_state();
            let c = nfa.push_state();
            let e = nfa.push_state();
            nfa.push_end(e);
            nfa.push_connection(StateId::of(0), a, from[0]);
            nfa.push_connection(a, b, from[1]);
            nfa.push_connection(b, c, from[2]);
            nfa.push_connections(c, e, from[3]..=191);
        }

        _ => {
            panic!("Not a valid start byte for a suposed utf-8 char")
        }
    }
}

impl<A: std::hash::Hash + Eq + Copy + std::fmt::Debug> From<&ClassUnicodeRange> for NFA<A> {
    fn from(range: &ClassUnicodeRange) -> Self {
        let mut start: [u8; 4] = [0; 4];
        range.start().encode_utf8(&mut start);
        let mut end: [u8; 4] = [0; 4];
        range.end().encode_utf8(&mut end);

        let start_len = range.start().len_utf8();
        let end_len = range.end().len_utf8();

        match (start_len, end_len) {
            (1, 1) => {
                //Ascii
                let mut nfa = NFA::<A>::empty();
                let e = nfa.push_state();
                nfa.push_connections(StateId::of(0), e, start[0]..=end[0]);
                nfa.push_end(e);
                nfa
            }
            (1, 2) => {
                let mut nfa = NFA::empty();
                more_or_eq_for_given_length(&mut nfa, start);
                below_or_eq_for_given_length(&mut nfa, end);
                nfa
            }
            (1, 3) => {
                let mut nfa = NFA::empty();
                more_or_eq_for_given_length(&mut nfa, start);
                any_char_of_length(&mut nfa, 2);
                below_or_eq_for_given_length(&mut nfa, end);
                nfa
            }
            (1, 4) => {
                let mut nfa = NFA::empty();
                more_or_eq_for_given_length(&mut nfa, start);
                any_char_of_length(&mut nfa, 2);
                any_char_of_length(&mut nfa, 3);
                below_or_eq_for_given_length(&mut nfa, end);
                nfa
            }
            (2, 1) => unreachable!("(2,1) case should be impossible"),
            (2, 2) => {
                match (start[0] == end[0], start[1] == end[1]) {
                    (true, true) => {
                        let mut nfa = NFA::<A>::empty();
                        let a = nfa.push_state();
                        let e = nfa.push_state();
                        nfa.push_end(e);
                        nfa.push_connection(StateId::of(0), a, start[0]);
                        nfa.push_connection(a, e, start[1]);
                        nfa
                    }
                    (true, false) => {
                        let mut nfa = NFA::<A>::empty();
                        let a = nfa.push_state();
                        let e = nfa.push_state();
                        nfa.push_end(e);
                        nfa.push_connection(StateId::of(0), a, start[0]);
                        nfa.push_connections(a, e, start[1]..=end[1]);
                        nfa
                    }
                    (false, true) | (false, false) => {
                        // Eq start[0]
                        let mut nfa = NFA::<A>::empty();
                        let a = nfa.push_state();
                        let e = nfa.push_state();
                        nfa.push_end(e);
                        nfa.push_connection(StateId::of(0), a, start[0]);
                        nfa.push_connections(a, e, start[1]..192);

                        // Eq end[0]
                        let a = nfa.push_state();
                        let e = nfa.push_state();
                        nfa.push_end(e);
                        nfa.push_connection(StateId::of(0), a, end[0]);
                        nfa.push_connections(a, e, 128..=end[1]);

                        // Between
                        let a = nfa.push_state();
                        let e = nfa.push_state();
                        nfa.push_end(e);
                        nfa.push_connections(StateId::of(0), a, start[0] + 1..end[0]);
                        nfa.push_connections(a, e, ONWARDS);

                        nfa
                    }
                }
            }
            (2, 3) => {
                let mut nfa = NFA::empty();
                more_or_eq_for_given_length(&mut nfa, start);
                below_or_eq_for_given_length(&mut nfa, end);
                nfa
            }
            (2, 4) => {
                let mut nfa = NFA::empty();
                more_or_eq_for_given_length(&mut nfa, start);
                any_char_of_length(&mut nfa, 3);
                below_or_eq_for_given_length(&mut nfa, end);
                nfa
            }
            (3, 1) => unreachable!("(3,1) case should be impossible"),
            (3, 2) => unreachable!("(3,2) case should be impossible"),
            (3, 3) => match (start[0] == end[0], start[1] == end[1], start[2] == end[2]) {
                (true, true, true) => {
                    let mut nfa = NFA::empty();
                    let a = nfa.push_state();
                    let b = nfa.push_state();
                    let e = nfa.push_state();
                    nfa.push_end(e);

                    nfa.push_connection(StateId::of(0), a, start[0]);
                    nfa.push_connection(a, b, start[1]);
                    nfa.push_connection(b, e, start[2]);
                    nfa
                }
                (true, true, false) => {
                    let mut nfa = NFA::empty();
                    let a = nfa.push_state();
                    let b = nfa.push_state();
                    let e = nfa.push_state();
                    nfa.push_end(e);

                    nfa.push_connection(StateId::of(0), a, start[0]);
                    nfa.push_connection(a, b, start[1]);
                    nfa.push_connections(b, e, start[2]..=end[2]);
                    nfa
                }
                (true, false, _) => {
                    let mut nfa = NFA::empty();
                    let a = nfa.push_state();
                    nfa.push_connection(StateId::of(0), a, start[0]);
                    between2(&mut nfa, [start[1], start[2]], [end[1], end[2]], a);
                    nfa
                }
                (false, _, _) => {
                    let mut nfa = NFA::empty();
                    between3(
                        &mut nfa,
                        [start[0], start[1], start[2]],
                        [end[0], end[1], end[2]],
                        StateId::of(0),
                    );
                    nfa
                }
            },
            (3, 4) => {
                let mut nfa = NFA::empty();
                more_or_eq_for_given_length(&mut nfa, start);
                below_or_eq_for_given_length(&mut nfa, end);
                nfa
            }
            (4, 4) => {
                match (
                    start[0] == end[0],
                    start[1] == end[1],
                    start[2] == end[2],
                    start[3] == end[3],
                ) {
                    (false, _, _, _) => {
                        let mut nfa = NFA::empty();

                        // First equal to start[0]
                        let a = nfa.push_state();
                        nfa.push_connection(StateId::of(0), a, start[0]);
                        // TODO 192 -> 191
                        between3(&mut nfa, [start[1], start[2], start[3]], [191, 191, 191], a);

                        // First between
                        if start[0] as usize + 1 < end[0] as usize {
                            let a = nfa.push_state();
                            nfa.push_connections(StateId::of(0), a, start[0] + 1..end[0]);
                            between3(&mut nfa, [128, 128, 128], [191, 191, 191], a);
                        }

                        // First equal to end[0]
                        let a = nfa.push_state();
                        nfa.push_connection(StateId::of(0), a, end[0]);
                        between3(&mut nfa, [128, 128, 128], [end[1], end[2], end[3]], a);

                        nfa
                    }
                    (true, false, _, _) => {
                        let mut nfa = NFA::empty();
                        let a = nfa.push_state();
                        nfa.push_connection(StateId::of(0), a, start[0]);
                        between3(
                            &mut nfa,
                            [start[1], start[2], start[3]],
                            [end[1], end[2], end[3]],
                            a,
                        );
                        nfa
                    }
                    (true, true, false, _) => {
                        let mut nfa = NFA::empty();
                        let a = nfa.push_state();
                        let b = nfa.push_state();

                        nfa.push_connection(StateId::of(0), a, start[0]);
                        nfa.push_connection(a, b, start[1]);

                        between2(&mut nfa, [start[2], start[3]], [end[2], end[3]], b);
                        nfa
                    }
                    (true, true, true, _) => {
                        let mut nfa = NFA::empty();
                        let a = nfa.push_state();
                        let b = nfa.push_state();
                        let c = nfa.push_state();
                        let e = nfa.push_state();
                        nfa.push_end(e);

                        nfa.push_connection(StateId::of(0), a, start[0]);
                        nfa.push_connection(a, b, start[1]);
                        nfa.push_connection(b, c, start[2]);
                        nfa.push_connections(c, e, start[3]..=end[3]);

                        nfa
                    }
                }
            }

            _ => unreachable!("_ case should be impossible"),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::regex::dfa::DFA;

    use super::*;

    fn is_between(from: char, x: char, to: char) -> bool {
        return (from as u32) <= (x as u32) && (x as u32) <= (to as u32);
    }

    #[quickcheck]
    fn qt_between2(from: char, to: char, other: char) -> bool {
        match (from.len_utf8(), to.len_utf8(), other.len_utf8()) {
            (2, 2, 2) => {
                if !(from < to) {
                    return true;
                }

                let mut from_buff = [0u8; 4];
                let mut to_buff = [0u8; 4];

                from.encode_utf8(&mut from_buff);
                to.encode_utf8(&mut to_buff);

                let mut nfa = NFA::<usize>::empty();
                between2(
                    &mut nfa,
                    [from_buff[0], from_buff[1]],
                    [to_buff[0], to_buff[1]],
                    StateId::of(0),
                );
                let dfa = DFA::from(nfa);
                let found = if !dfa.find(other.to_string().as_str()).is_ok() {
                    let mut value_buff = [0u8; 4];
                    other.encode_utf8(&mut value_buff);
                    println!(
                        "from:{}:{:?},  to:{}:{:?}  value:{}:{:?}",
                        from,
                        from_buff.to_vec(),
                        to,
                        to_buff.to_vec(),
                        other,
                        value_buff.to_vec(),
                    );

                    false
                } else {
                    true
                };

                found == is_between(from, other, to)
            }
            _ => true,
        }
    }

    #[quickcheck]
    fn qt_between3(from: char, to: char, other: char) -> bool {
        match (from.len_utf8(), to.len_utf8(), other.len_utf8()) {
            (3, 3, 3) => {
                if !(from < to) {
                    return true;
                }

                let mut from_buff = [0u8; 4];
                let mut to_buff = [0u8; 4];

                from.encode_utf8(&mut from_buff);
                to.encode_utf8(&mut to_buff);

                let mut nfa = NFA::<usize>::empty();
                between3(
                    &mut nfa,
                    [from_buff[0], from_buff[1], from_buff[2]],
                    [to_buff[0], to_buff[1], to_buff[2]],
                    StateId::of(0),
                );
                let dfa = DFA::from(nfa);
                let found = if !dfa.find(other.to_string().as_str()).is_ok() {
                    let mut value_buff = [0u8; 4];
                    other.encode_utf8(&mut value_buff);
                    println!(
                        "from:{}:{:?},  to:{}:{:?}  value:{}:{:?}",
                        from,
                        from_buff.to_vec(),
                        to,
                        to_buff.to_vec(),
                        other,
                        value_buff.to_vec(),
                    );

                    false
                } else {
                    true
                };

                found == is_between(from, other, to)
            }
            _ => true,
        }
    }

    #[quickcheck]
    fn qc_utf8_range(from: char, to: char, other: char) -> bool {
        if !is_between(from, other, to) {
            return true;
        }

        let class = ClassUnicodeRange::new(from, to);
        let nfa = NFA::<usize>::from(&class);

        if !(nfa._find(other.to_string().as_str()).is_ok()) {
            let mut from_buff = [0u8; 4];
            from.encode_utf8(&mut from_buff);
            let mut to_buff = [0u8; 4];
            to.encode_utf8(&mut to_buff);
            let mut value_buff = [0u8; 4];
            other.encode_utf8(&mut value_buff);
            println!(
                "from:{}:{:?},  to:{}:{:?}  value:{}:{:?}",
                from,
                from_buff.to_vec(),
                to,
                to_buff.to_vec(),
                other,
                value_buff.to_vec()
            );
            return false;
        }
        true
    }

    // #[test]
    // fn test_between2() {
    //     for from1 in &[128, 129, 190, 191] {
    //         for from2 in &[128, 129, 130, 131, 132, 189, 190, 191] {
    //             for to1 in &[128, 129, 130, 189, 190, 191] {
    //                 for to2 in ONWARDS {
    //                     let from = [230u8, *from1, *from2];
    //                     let from = std::str::from_utf8(&from).unwrap();
    //                     let from = from.chars().next().unwrap();
    //                     println!("{}", from);

    //                     let to = [230u8, *to1, to2];
    //                     let to = std::str::from_utf8(&to).unwrap();
    //                     let to = to.chars().next().unwrap();

    //                     if (to as u32) < (from as u32) {
    //                         continue;
    //                     }

    //                     let mut nfa = NFA::<usize>::empty();
    //                     let a = nfa.push_state();
    //                     nfa.push_connection(StateId::of(0), a, 230);
    //                     between2(&mut nfa, [*from1, *from2], [*to1, to2], a);
    //                     let dfa = DFA::from(nfa);

    //                     for value1 in 128..192 {
    //                         for value2 in 128..192 {
    //                             let value = [230u8, value1, value2];
    //                             let value = std::str::from_utf8(&value).unwrap();
    //                             let value = value.chars().next().unwrap();

    //                             if is_between(from, value, to) || is_between(to, value, from) {
    //                                 if !(dfa.find(value.to_string().as_str()).is_ok()) {
    //                                     let mut from_buff = [0u8; 4];
    //                                     from.encode_utf8(&mut from_buff);
    //                                     let mut to_buff = [0u8; 4];
    //                                     to.encode_utf8(&mut to_buff);
    //                                     let mut value_buff = [0u8; 4];
    //                                     value.encode_utf8(&mut value_buff);
    //                                     println!("{:?}", dfa);
    //                                     println!(
    //                                         "from:{}:{:?},  to:{}:{:?}  value:{}:{:?}",
    //                                         from,
    //                                         from_buff.to_vec(),
    //                                         to,
    //                                         to_buff.to_vec(),
    //                                         value,
    //                                         value_buff.to_vec(),
    //                                     );

    //                                     println!(
    //                                         "ERROR from:{} to:{} value:{} should be accepted",
    //                                         from, to, value
    //                                     );

    //                                     std::io::Write::flush(&mut std::io::stdout()).unwrap();

    //                                     assert!(false);
    //                                 }
    //                             } else {
    //                                 if !(dfa.find(value.to_string().as_str()).is_err()) {
    //                                     let mut from_buff = [0u8; 4];
    //                                     from.encode_utf8(&mut from_buff);
    //                                     let mut to_buff = [0u8; 4];
    //                                     to.encode_utf8(&mut to_buff);
    //                                     let mut value_buff = [0u8; 4];
    //                                     value.encode_utf8(&mut value_buff);

    //                                     println!(
    //                                         "from:{}:{:?},  to:{}:{:?}  value:{}:{:?}",
    //                                         from,
    //                                         from_buff.to_vec(),
    //                                         to,
    //                                         to_buff.to_vec(),
    //                                         value,
    //                                         value_buff.to_vec(),
    //                                     );

    //                                     println!(
    //                                         "ERROR from:{} to:{} value:{} should fail but does not",
    //                                         from, to, value
    //                                     );
    //                                     std::io::Write::flush(&mut std::io::stdout()).unwrap();
    //                                     assert!(false);
    //                                 }
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }

    //#[test]
    // fn test_between3() {
    //     for from1 in &[128, 129, 190, 191] {
    //         for from2 in &[128, 129, 130, 190, 191] {
    //             for from3 in &[128, 129, 130, 131, 190, 191] {
    //                 for to1 in &[128, 129, 130, 189, 190, 191] {
    //                     for to2 in &[128, 129, 130, 131, 187, 190, 191] {
    //                         for to3 in &[128, 129, 130, 131, 150, 190, 191] {
    //                             let from = [241u8, *from1, *from2, *from3];
    //                             let from = std::str::from_utf8(&from).unwrap();
    //                             let from = from.chars().next().unwrap();
    //                             println!("{}", from);

    //                             let to = [241u8, *to1, *to2, *to3];
    //                             let to = std::str::from_utf8(&to).unwrap();
    //                             let to = to.chars().next().unwrap();

    //                             if (to as u32) < (from as u32) {
    //                                 continue;
    //                             }

    //                             // let mut nfa = NFA::<usize>::empty();
    //                             // let a = nfa.push_state();
    //                             // nfa.push_connection(StateId::of(0), a, 241);
    //                             // between3(&mut nfa, [*from1, *from2, *from3], [*to1, *to2, *to3], a);
    //                             let class = ClassUnicodeRange::new(from, to);
    //                             let nfa = NFA::<usize>::from(&class);
    //                             //let nfa = NFA::from();
    //                             let dfa = DFA::from(nfa);

    //                             for value1 in 128..192 {
    //                                 for value2 in 128..192 {
    //                                     for value3 in 128..192 {
    //                                         let value = [241u8, value1, value2, value3];
    //                                         let value = std::str::from_utf8(&value).unwrap();
    //                                         let value = value.chars().next().unwrap();

    //                                         if is_between(from, value, to)
    //                                             || is_between(to, value, from)
    //                                         {
    //                                             if !(dfa.find(value.to_string().as_str()).is_ok()) {
    //                                                 let mut from_buff = [0u8; 4];
    //                                                 from.encode_utf8(&mut from_buff);
    //                                                 let mut to_buff = [0u8; 4];
    //                                                 to.encode_utf8(&mut to_buff);
    //                                                 let mut value_buff = [0u8; 4];
    //                                                 value.encode_utf8(&mut value_buff);
    //                                                 println!("{:?}", dfa);
    //                                                 println!(
    //                                                     "from:{}:{:?},  to:{}:{:?}  value:{}:{:?}",
    //                                                     from,
    //                                                     from_buff.to_vec(),
    //                                                     to,
    //                                                     to_buff.to_vec(),
    //                                                     value,
    //                                                     value_buff.to_vec(),
    //                                                 );

    //                                                 println!(
    //                                             "ERROR from:{} to:{} value:{} should be accepted",
    //                                             from, to, value
    //                                         );

    //                                                 std::io::Write::flush(&mut std::io::stdout())
    //                                                     .unwrap();

    //                                                 assert!(false);
    //                                             }
    //                                         } else {
    //                                             if !(dfa.find(value.to_string().as_str()).is_err())
    //                                             {
    //                                                 let mut from_buff = [0u8; 4];
    //                                                 from.encode_utf8(&mut from_buff);
    //                                                 let mut to_buff = [0u8; 4];
    //                                                 to.encode_utf8(&mut to_buff);
    //                                                 let mut value_buff = [0u8; 4];
    //                                                 value.encode_utf8(&mut value_buff);

    //                                                 println!(
    //                                                     "from:{}:{:?},  to:{}:{:?}  value:{}:{:?}",
    //                                                     from,
    //                                                     from_buff.to_vec(),
    //                                                     to,
    //                                                     to_buff.to_vec(),
    //                                                     value,
    //                                                     value_buff.to_vec(),
    //                                                 );

    //                                                 println!(
    //                                             "ERROR from:{} to:{} value:{} should fail but does not",
    //                                             from, to, value
    //                                         );
    //                                                 std::io::Write::flush(&mut std::io::stdout())
    //                                                     .unwrap();
    //                                                 assert!(false);
    //                                             }
    //                                         }
    //                                     }
    //                                 }
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }

    #[test]
    // fn test_failing() {
    //     let from = '𿾿';
    //     let to = '򟘈';
    //     let value = '𿿿';

    //     let mut from_buff = [0u8; 4];
    //     let mut to_buff = [0u8; 4];

    //     from.encode_utf8(&mut from_buff);
    //     to.encode_utf8(&mut to_buff);

    //     let class = ClassUnicodeRange::new(from, to);
    //     let nfa = NFA::<usize>::from(&class);
    //     let dfa = DFA::from(nfa);

    //     if is_between(from, value, to) || is_between(to, value, from) {
    //         if !(dfa.find(value.to_string().as_str()).is_ok()) {
    //             let mut from_buff = [0u8; 4];
    //             from.encode_utf8(&mut from_buff);
    //             let mut to_buff = [0u8; 4];
    //             to.encode_utf8(&mut to_buff);
    //             let mut value_buff = [0u8; 4];
    //             value.encode_utf8(&mut value_buff);

    //             println!(
    //                 "from:{}:{:?},  to:{}:{:?}  value:{}:{:?}",
    //                 from,
    //                 from_buff.to_vec(),
    //                 to,
    //                 to_buff.to_vec(),
    //                 value,
    //                 value_buff.to_vec(),
    //             );

    //             println!(
    //                 "ERROR from:{} to:{} value:{} should be accepted but is not",
    //                 from, to, value
    //             );

    //             std::io::Write::flush(&mut std::io::stdout()).unwrap();

    //             assert!(false);
    //         }
    //     } else {
    //         if !(dfa.find(value.to_string().as_str()).is_err()) {
    //             let mut from_buff = [0u8; 4];
    //             from.encode_utf8(&mut from_buff);
    //             let mut to_buff = [0u8; 4];
    //             to.encode_utf8(&mut to_buff);
    //             let mut value_buff = [0u8; 4];
    //             value.encode_utf8(&mut value_buff);

    //             println!(
    //                 "from:{}:{:?},  to:{}:{:?}  value:{}:{:?}",
    //                 from,
    //                 from_buff.to_vec(),
    //                 to,
    //                 to_buff.to_vec(),
    //                 value,
    //                 value_buff.to_vec(),
    //             );

    //             println!(
    //                 "ERROR from:{} to:{} value:{} should fail but does not",
    //                 from, to, value
    //             );
    //             std::io::Write::flush(&mut std::io::stdout()).unwrap();
    //             assert!(false);
    //         }
    //     }
    // }
    #[test]
    fn test_all() {
        let test_cases = [
            // ascii
            'a', 'b', 'c', 'd', 'z', // 2byte
            'ϱ', 'ϯ', 'Ϯ', 'ϯ', 'ϰ', 'ΰ', 'ί', 'ή', 'έ', 'Ͱ', 'ͱ', 'Ͳ', 'Ͱ', 'ඌ', //3 byte
            'ଵ', 'ଶ', 'ଷ', 'ସ', '୵', '୶', '୷', 'வ', 'ஶ', 'ஷ', 'ᬵ', '㬵', '㭵', '㮵', '㯵', '㬶',
            '㬷', '䬷', //4 byte
            '😈', '😉', '😊', '🙈', '🚈', '🚉', '🙉', '🙇', '😌', '򟘈', '󟘈', '󜘈', '󄖄',
            // edgecase ascii
            '', '', '}', '~', '', // edgecase 2byte
            'À', 'Á', 'Â', 'Ã', 'ß', 'Þ', 'Ý', '߿', '߾', '޿', // edgecase 3byte
            'က', 'ခ', 'ဂ', '၀', 'ႀ', // edgecase 4 byte
            '񀀀', '𿿿', '𿿾', '𿾿', '𾿿', '󿿿',
        ];
        let mut num = 0;
        for from in test_cases.iter() {
            for to in test_cases.iter() {
                if from > to {
                    continue;
                }

                num += 1;

                let range = ClassUnicodeRange::new(*from, *to);
                let nfa = NFA::<usize>::from(&range);
                let dfa = DFA::<usize>::from(nfa);

                for value in test_cases.iter() {
                    let string = value.to_string();

                    if is_between(*from, *value, *to) || is_between(*to, *value, *from) {
                        if !(dfa.find(&string).is_ok()) {
                            let mut from_buff = [0u8; 4];
                            from.encode_utf8(&mut from_buff);
                            let mut to_buff = [0u8; 4];
                            to.encode_utf8(&mut to_buff);
                            let mut value_buff = [0u8; 4];
                            value.encode_utf8(&mut value_buff);

                            println!(
                                "from:{}:{:?},  to:{}:{:?}  value:{}:{:?}, num:{}",
                                from,
                                from_buff.to_vec(),
                                to,
                                to_buff.to_vec(),
                                value,
                                value_buff.to_vec(),
                                num
                            );

                            println!(
                                "ERROR from:{} to:{} value:{} should be accepted",
                                from, to, value
                            );
                            assert!(false);
                        }
                    } else {
                        if !(dfa.find(&string).is_err()) {
                            let mut from_buff = [0u8; 4];
                            from.encode_utf8(&mut from_buff);
                            let mut to_buff = [0u8; 4];
                            to.encode_utf8(&mut to_buff);
                            let mut value_buff = [0u8; 4];
                            value.encode_utf8(&mut value_buff);

                            println!(
                                "from:{}:{:?},  to:{}:{:?}  value:{}:{:?}, num:{}",
                                from,
                                from_buff.to_vec(),
                                to,
                                to_buff.to_vec(),
                                value,
                                value_buff.to_vec(),
                                num
                            );

                            println!(
                                "ERROR from:{} to:{} value:{} should fail but does not",
                                from, to, value
                            );
                            assert!(false);
                        }
                    }
                }
            }
        }
    }
}
