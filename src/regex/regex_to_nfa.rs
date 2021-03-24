use std::ops::Range;

use regex_syntax::Parser;

use anyhow::{bail, Result};

use crate::regex::NFA;

fn regex_to_nfa<A: std::hash::Hash + Eq + Copy + std::fmt::Debug>(regex: &str) -> Result<NFA<A>> {
    let hir = Parser::new().parse(regex)?;
    hir_to_nfa(&hir)
}

fn hir_to_nfa<A: std::hash::Hash + Eq + Copy + std::fmt::Debug>(
    hir: &regex_syntax::hir::Hir,
) -> Result<NFA<A>> {
    match hir.kind() {
        regex_syntax::hir::HirKind::Empty => Ok(NFA::literal("")),
        regex_syntax::hir::HirKind::Literal(lit) => match lit {
            regex_syntax::hir::Literal::Unicode(uni) => Ok(NFA::literal(&uni.to_string())),
            regex_syntax::hir::Literal::Byte(byte) => Ok(NFA::literal(&byte.to_string())),
        },
        regex_syntax::hir::HirKind::Class(class) => {
            match class {
                regex_syntax::hir::Class::Unicode(uni) => {
                    let mut nfa = NFA::<A>::empty();
                    for range in uni.ranges() {
                        nfa = nfa.or(NFA::<A>::from(range))?;
                    }
                    Ok(nfa)
                }
                regex_syntax::hir::Class::Bytes(byte) => {
                    let mut nfa = NFA::empty();
                    for range in byte.iter() {
                        //Todo check that range is inclusive
                        nfa = nfa.or(NFA::from(Range {
                            start: range.start(),
                            end: range.end(),
                        }))?;
                    }
                    Ok(nfa)
                }
            }
        }
        regex_syntax::hir::HirKind::Anchor(x) => match x {
            regex_syntax::hir::Anchor::StartLine => bail!("We dont suport StartLine symbols!"),
            regex_syntax::hir::Anchor::EndLine => bail!("We dont suport EndLine symbols!"),
            regex_syntax::hir::Anchor::StartText => bail!("We dont suport StartText symbol!"),
            regex_syntax::hir::Anchor::EndText => bail!("We dont suport EndText symbol!"),
        },
        regex_syntax::hir::HirKind::WordBoundary(boundary) => {
            match boundary {
                regex_syntax::hir::WordBoundary::Unicode => {
                    todo!() // I dont know if we need to suport this
                }
                regex_syntax::hir::WordBoundary::UnicodeNegate => {
                    todo!() // I dont know if we need to suport this
                }
                regex_syntax::hir::WordBoundary::Ascii => {
                    todo!() // I dont know if we need to suport this
                }
                regex_syntax::hir::WordBoundary::AsciiNegate => {
                    todo!() // I dont know if we need to suport this
                }
            }
        }
        regex_syntax::hir::HirKind::Repetition(x) => {
            if x.greedy {
                let nfa = hir_to_nfa(&x.hir)?;
                Ok(nfa.repeat()?)
            } else {
                bail!("We dont suport non greedy patterns")
            }
        }
        regex_syntax::hir::HirKind::Group(group) => {
            //TODO i dont know how we are suposed to interprite an empty
            //hir/nfa in this case. Should it maybe be a no-op?
            hir_to_nfa(&group.hir)
        }
        regex_syntax::hir::HirKind::Concat(cats) => {
            let mut nfas = cats.iter().map(|hir| hir_to_nfa(hir));
            let mut fst = nfas.next().unwrap()?;
            for nfa in nfas {
                fst.followed_by(nfa?)?;
            }
            Ok(fst)
        }

        regex_syntax::hir::HirKind::Alternation(alts) => {
            let mut nfas = alts.iter().map(|hir| hir_to_nfa(hir));
            let mut fst = nfas.next().unwrap()?;
            for nfa in nfas {
                fst = fst.or(nfa?)?;
            }
            Ok(fst)
        }
    }
}

impl<A: std::hash::Hash + Eq + Copy + std::fmt::Debug> NFA<A> {
    pub fn regex(string: &str) -> anyhow::Result<NFA<A>> {
        regex_to_nfa(string)
    }
}

#[cfg(test)]
mod tests {

    use crate::regex::dfa::DFA;

    use super::*;
    #[test]
    fn simple1() {
        let nfa = NFA::<usize>::regex("fu.*").unwrap();
        let dfa = DFA::<usize>::from(nfa);

        for case in &["funN", "fu.\"", "fu,-", "fu{:", "fut!"] {
            assert!(dfa.find(case).is_ok());
        }
    }

    #[test]
    fn simple2() {
        let nfa = NFA::<usize>::regex("fu..*").unwrap();
        let dfa = DFA::<usize>::from(nfa);

        for case in &["funN", "fu.\"", "fu,-", "fu{:", "fut!"] {
            assert!(dfa.find(case).is_ok());
        }

        for case in &["fu"] {
            assert!(dfa.find(case).is_err());
        }
    }

    #[test]
    fn digit() {
        let nfa = NFA::<usize>::regex("\\d").unwrap();
        let dfa = DFA::<usize>::from(nfa);

        for case in &["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"] {
            assert!(dfa.find(case).is_ok());
        }
        for case in &["a"] {
            assert!(dfa.find(case).is_err());
        }
    }

    #[test]
    fn not_digit() {
        let nfa = NFA::<usize>::regex("\\D").unwrap();
        let dfa = DFA::<usize>::from(nfa);

        for case in &["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"] {
            assert!(dfa.find(case).is_err());
        }
        for case in &["a", "q"] {
            assert!(dfa.find(case).is_ok());
        }
    }

    #[test]
    fn direct_sibtraction() {
        let nfa = NFA::<usize>::regex("[0-9--4]").unwrap();
        let dfa = DFA::<usize>::from(nfa);

        for case in &["1", "2", "3", "5", "6", "7", "8", "9", "0"] {
            assert!(dfa.find(case).is_ok());
        }
        for case in &["4", "a"] {
            assert!(dfa.find(case).is_err());
        }
    }
}
