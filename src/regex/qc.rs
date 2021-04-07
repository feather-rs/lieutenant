#[cfg(test)]
use crate::regex::NFA;
#[cfg(test)]
use quickcheck::Arbitrary;
#[cfg(test)]
use std::{collections::HashSet, iter};
#[cfg(test)]
const MAX_LIT_LEN: usize = 2; // How many chars the string literals should be.
#[cfg(test)]
const DEBTH: usize = 5; // How deep the nfa should be.

/*
This file contains code for generating testcases for quickcheck.
*/
#[cfg(test)]
#[derive(Debug, Clone)]
pub enum NfaQt {
    Lit(String),
    Or { a: Box<NfaQt>, b: Box<NfaQt> },
    FollowedBy { a: Box<NfaQt>, b: Box<NfaQt> },
}

#[cfg(test)]
impl NfaQt {
    fn new(g: &mut quickcheck::Gen, level: usize) -> Self {
        if level == 0 {
            return NfaQt::Lit(
                String::arbitrary(g)
                    .chars()
                    .into_iter()
                    .take(MAX_LIT_LEN)
                    .collect(),
            );
        }

        let choice = g.choose(&[0, 1]).unwrap();
        match choice {
            0 => NfaQt::Or {
                a: Box::new(NfaQt::new(g, level - 1)),
                b: Box::new(NfaQt::new(g, level - 1)),
            },
            1 => NfaQt::FollowedBy {
                a: Box::new(NfaQt::new(g, level - 1)),
                b: Box::new(NfaQt::new(g, level - 1)),
            },
            _ => unreachable!("not a valid choice"),
        }
    }

    #[cfg(test)]
    fn build_matches(&self) -> HashSet<String> {
        match self {
            NfaQt::Lit(x) => iter::once(x.to_string()).collect(),
            NfaQt::Or { a, b } => {
                let mut res = a.build_matches();
                res.extend(b.build_matches());
                res
            }
            NfaQt::FollowedBy { a, b } => {
                let abm = a.build_matches();
                let bbm = b.build_matches();
                let mut res = HashSet::with_capacity(abm.len() * bbm.len());
                for am in abm.iter() {
                    for bm in bbm.iter() {
                        let mut s = String::with_capacity(am.len() + bm.len());
                        s.push_str(am.as_str());
                        s.push_str(bm.as_str());
                        res.insert(s);
                    }
                }
                res
            }
        }
    }
    #[cfg(test)]
    fn build_nfa(&self) -> NFA<usize> {
        match self {
            NfaQt::Lit(x) => NFA::literal(x),
            NfaQt::Or { a, b } => {
                let nfa = a.build_nfa();
                let nfb = b.build_nfa();
                nfa.or(nfb).unwrap()
            }
            NfaQt::FollowedBy { a, b } => {
                let mut nfa = a.build_nfa();
                let nfb = b.build_nfa();
                nfa.followed_by(nfb).unwrap();
                nfa
            }
        }
    }
}

#[cfg(test)]
#[derive(Debug, Clone)]
pub struct NFAQtCase {
    pub kind: NfaQt,
    pub matches: HashSet<String>,
    pub nfa: NFA<usize>,
}

#[cfg(test)]
impl Arbitrary for NFAQtCase {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let limit = DEBTH;
        let mut level = g.size();
        level = if level <= limit { level } else { limit };

        let kind = NfaQt::new(g, level);
        let matches = kind.build_matches();
        let nfa = kind.build_nfa();

        Self { kind, matches, nfa }
    }
}
