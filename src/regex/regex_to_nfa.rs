use std::ops::Range;

use regex_syntax::Parser;

use anyhow::{bail, Result};

use crate::regex::NFA;

fn regex_to_nfa<A: std::hash::Hash + Eq + Copy + std::fmt::Debug>(regex: &str) -> Result<NFA<A>> {
    let hir = Parser::new().parse(regex)?;
    hir_to_nfa(&hir)
}

fn repeated_n_times<A>(mut nfa: NFA<A>, n: u32) -> anyhow::Result<NFA<A>>
where
    A: std::hash::Hash + Eq + Copy + std::fmt::Debug,
{
    let mut result = NFA::<A>::empty();
    let mut accounted = 0_u32;

    for bit_index in 0..=31 {
        if accounted == n {
            break;
        }

        if n & (1 << bit_index) != 0 {
            println!("bit_index match {}", bit_index);
            result.followed_by(nfa.clone())?;
            accounted |= 1 << bit_index;
        }

        // Double nfa in size
        nfa.followed_by(nfa.clone())?;
    }

    Ok(result)
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
            match &x.kind {
                regex_syntax::hir::RepetitionKind::ZeroOrOne => {
                    let nfa = NFA::<A>::empty();
                    nfa.or(hir_to_nfa(&x.hir)?)
                }
                regex_syntax::hir::RepetitionKind::ZeroOrMore => hir_to_nfa(&x.hir)?.repeat(),
                regex_syntax::hir::RepetitionKind::OneOrMore => {
                    let nfa = hir_to_nfa(&x.hir)?;
                    let mut fst = nfa.clone();
                    fst.followed_by(nfa.repeat()?)?;
                    Ok(fst)
                }
                regex_syntax::hir::RepetitionKind::Range(range) => match range {
                    // We dont care about greedy vs lazy ranges, since we only use the regex to detect
                    // fullmatch, and dont care about matchgroups.
                    regex_syntax::hir::RepetitionRange::Exactly(exact) => {
                        println!("Exact branch");
                        let nfa = hir_to_nfa(&x.hir)?;
                        repeated_n_times(nfa, *exact)
                    }
                    regex_syntax::hir::RepetitionRange::AtLeast(exact) => {
                        println!("AT LEAST BRANCH");
                        let nfa = hir_to_nfa(&x.hir)?;
                        let mut result = repeated_n_times(nfa.clone(), *exact)?;
                        result.followed_by(nfa.repeat()?)?;
                        Ok(result)
                    }
                    regex_syntax::hir::RepetitionRange::Bounded(n, m) => {
                        let mut result = NFA::empty();
                        let org = hir_to_nfa(&x.hir)?;
                        let mut nfa = repeated_n_times::<A>(org.clone(), *n)?;

                        for _ in *n..=*m {
                            result = result.or(nfa.clone())?;
                            nfa.followed_by(org.clone())?;
                        }

                        Ok(result)
                    }
                },
            }
        }
        regex_syntax::hir::HirKind::Group(group) => {
            match &group.kind {
                regex_syntax::hir::GroupKind::CaptureIndex(_) => hir_to_nfa(&group.hir),
                regex_syntax::hir::GroupKind::NonCapturing => {
                    hir_to_nfa(&group.hir)
                }
                regex_syntax::hir::GroupKind::CaptureName { name: _, index: _ } => {
                    hir_to_nfa(&group.hir)
                }
            }
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

pub enum RegexConvertError {
    StartLine,
    EndLine,
    StartText,
    EndText,
    WordBoundaryUnicode,
    WordBoundaryUnicodeNegate,
    WordBoundaryAscii,
    WordBoundaryAsciiNegate,
    RegexParseError(regex_syntax::Error),
}

impl std::fmt::Debug for RegexConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            RegexConvertError::StartLine => {
                write!(f,"We don't suprt start line in regex, so ^ is not allowed in the regex without escaping it with \\.")
            }
            RegexConvertError::EndLine => {
                write!(f,"We don't suport end line in regex, so $ is not allowed in the regex  without escaping it with \\.")
            }
            RegexConvertError::StartText => {
                write!(f,"We don't suport start text symbol in regex, so \\A is not allowed in the regex.")
            }
            RegexConvertError::EndText => {
                write!(f,"We don't suport end of text symbol in regex, so \\z is not allowed in the regex.")
            }
            RegexConvertError::WordBoundaryUnicode => {
                write!(
                    f,
                    "We don't suport unicode world boundary, so \\b is not allowed in the regex."
                )
            }
            RegexConvertError::WordBoundaryUnicodeNegate => {
                write!(f,"We don't suport \"not a unicode world boundary\", so \\B is not allowed in the regex.")
            }
            RegexConvertError::WordBoundaryAscii => {
                write!(f,"We don't suport \"not a unicode world boundary\", so (?-u:\\b) is not allowed in the regex.")
            }
            RegexConvertError::WordBoundaryAsciiNegate => {
                write!(f,"We don't suport \"not a unicode world boundary\", so (?-u:\\B) is not allowed in the regex.")
            }

            RegexConvertError::RegexParseError(err) => err.fmt(f),
        }
    }
}

/// Checks if regex contains a feature we don't suport, or it just cant be parsed as
/// valid regex. If this test passes for a command, then the only other failurecase for 
/// creating a nfa is running out of StateId: u32. 
pub fn we_suport_regex(regex: &str) -> Result<(), RegexConvertError> {
    let hir = Parser::new().parse(regex);
    let hir = match hir {
        Ok(x) => x,
        Err(e) => return Err(RegexConvertError::RegexParseError(e)),
    };

    we_suport_hir(&hir)
}

/// Returns wheter or not we are able to parse regex. We dont suport certain features.
fn we_suport_hir(hir: &regex_syntax::hir::Hir) -> Result<(), RegexConvertError> {
    match hir.kind() {
        regex_syntax::hir::HirKind::Empty => Ok(()),
        regex_syntax::hir::HirKind::Literal(lit) => match lit {
            regex_syntax::hir::Literal::Unicode(_) => Ok(()),
            regex_syntax::hir::Literal::Byte(_) => Ok(()),
        },
        regex_syntax::hir::HirKind::Class(class) => match class {
            regex_syntax::hir::Class::Unicode(_) => Ok(()),
            regex_syntax::hir::Class::Bytes(_) => Ok(()),
        },
        regex_syntax::hir::HirKind::Anchor(x) => match x {
            regex_syntax::hir::Anchor::StartLine => Err(RegexConvertError::StartLine),
            regex_syntax::hir::Anchor::EndLine => Err(RegexConvertError::EndLine),
            regex_syntax::hir::Anchor::StartText => Err(RegexConvertError::StartText),
            regex_syntax::hir::Anchor::EndText => Err(RegexConvertError::EndText),
        },
        regex_syntax::hir::HirKind::WordBoundary(boundary) => match boundary {
            regex_syntax::hir::WordBoundary::Unicode => Err(RegexConvertError::WordBoundaryUnicode),
            regex_syntax::hir::WordBoundary::UnicodeNegate => {
                Err(RegexConvertError::WordBoundaryUnicodeNegate)
            }
            regex_syntax::hir::WordBoundary::Ascii => Err(RegexConvertError::WordBoundaryAscii),
            regex_syntax::hir::WordBoundary::AsciiNegate => {
                Err(RegexConvertError::WordBoundaryAsciiNegate)
            }
        },
        regex_syntax::hir::HirKind::Repetition(x) => {
            match &x.kind {
                regex_syntax::hir::RepetitionKind::ZeroOrOne => Ok(()),
                regex_syntax::hir::RepetitionKind::ZeroOrMore => Ok(()),
                regex_syntax::hir::RepetitionKind::OneOrMore => Ok(()),
                regex_syntax::hir::RepetitionKind::Range(range) => match range {
                    // We dont care about greedy vs lazy ranges, since we only use the regex to detect
                    // fullmatch, and dont care about matchgroups.
                    regex_syntax::hir::RepetitionRange::Exactly(_) => Ok(()),
                    regex_syntax::hir::RepetitionRange::AtLeast(_) => Ok(()),
                    regex_syntax::hir::RepetitionRange::Bounded(_, _) => Ok(()),
                },
            }
        }
        regex_syntax::hir::HirKind::Group(group) => match &group.kind {
            regex_syntax::hir::GroupKind::CaptureIndex(_)
            | regex_syntax::hir::GroupKind::NonCapturing
            | regex_syntax::hir::GroupKind::CaptureName { name: _, index: _ } => Ok(()),
        },
        regex_syntax::hir::HirKind::Concat(cats) => {
            for meow in cats {
                if let Err(x) = we_suport_hir(meow) {
                    return Err(x);
                }
            }
            Ok(())
        }
        regex_syntax::hir::HirKind::Alternation(alts) => {
            for alt in alts {
                if let Err(x) = we_suport_hir(alt) {
                    return Err(x);
                }
            }
            Ok(())
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

    use super::*;
    use crate::regex::dfa::DFA;

    #[test]
    fn test_we_dont_suport() {
        let regex = "^";
        assert!(we_suport_regex(regex).is_err());

        let regex = "$";
        assert!(we_suport_regex(regex).is_err());

        let regex = "\\A";
        assert!(we_suport_regex(regex).is_err());

        let regex = "\\z";
        assert!(we_suport_regex(regex).is_err());

        let regex = "\\b";
        assert!(we_suport_regex(regex).is_err());


    }

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

    #[test]
    fn repetitions_exact() {
        let regex = "a{5}";
        let nfa = NFA::<usize>::regex(regex).unwrap();
        let dfa: DFA<usize> = nfa.into();
        //println!("{:?}",dfa);
        assert!(dfa.find("aaaaa").is_ok());
        assert!(dfa.find("aaaa").is_err());
        assert!(dfa.find("aaaaaa").is_err());
        assert!(dfa.find("").is_err());
    }

    #[test]
    fn repetitions_at_least() {
        let regex = "a{5,}";
        let nfa = NFA::<usize>::regex(regex).unwrap();
        let dfa: DFA<usize> = nfa.into();
        assert!(dfa.find("aaaaa").is_ok());
        assert!(dfa.find("aaaa").is_err());
        assert!(dfa.find("aaaaaa").is_ok());
        assert!(dfa.find("").is_err());
    }

    #[test]
    fn repetitions_between() {
        let regex = "a{5,8}";
        let nfa = NFA::<usize>::regex(regex).unwrap();
        let dfa: DFA<usize> = nfa.into();
        assert!(dfa.find("aaaaa").is_ok());
        assert!(dfa.find("aaaa").is_err());
        assert!(dfa.find("aaaaaa").is_ok());
        assert!(dfa.find("aaaaaaa").is_ok());
        assert!(dfa.find("aaaaaaaa").is_ok());
        assert!(dfa.find("aaaaaaaaa").is_err());
        assert!(dfa.find("").is_err());
    }

    #[test]
    fn repetition() {
        let mut nfa = NFA::<usize>::regex("ho").unwrap();
        nfa = nfa.repeat().unwrap();
        let dfa: DFA<usize> = nfa.into();

        assert!(dfa.find("").is_ok());
        assert!(dfa.find("ho").is_ok());
        assert!(dfa.find("hoho").is_ok());
        assert!(dfa.find("h").is_err());
    }

    #[test]
    fn repetition_lazy() {
        let nfa = NFA::<usize>::regex("a{3,5}?").unwrap();
        let dfa: DFA<usize> = nfa.into();

        assert!(dfa.find("aaa").is_ok());
        assert!(dfa.find("aaaa").is_ok());
        assert!(dfa.find("aaaaa").is_ok());
        assert!(dfa.find("").is_err());
    }
}
