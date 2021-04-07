use super::IterParser;
use anyhow::{anyhow, Result};

/// A literal should not have leading or trailing whitespaces.
pub struct Literal {
    pub(crate) value: String,
}

impl Literal {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

impl IterParser for Literal {
    type Extract = ();
    type ParserState = ();

    #[allow(clippy::type_complexity)]
    fn parse<'i>(
        &self,
        _state: Self::ParserState,
        input: &'i str,
    ) -> (Result<(Self::Extract, &'i str)>, Option<Self::ParserState>) {
        let input_lower = &mut input
            .trim_start()
            .chars()
            .flat_map(|c| c.to_lowercase())
            .peekable();
        let literal_lower = &mut self.value.chars().flat_map(|c| c.to_lowercase()).peekable();
        let mut ofsett = 0;

        loop {
            match (literal_lower.next(), input_lower.next()) {
                (None, None) => {
                    // Then the two str had the same length and were identical up to that point
                    return (Ok(((), &"")), None);
                }
                (None, Some(' ')) => {
                    // We have reached the end of the literal, and the next input char is a space
                    return (Ok(((), &input[ofsett..])), None);
                }
                (None, Some(c)) => {
                    // We have reached the end of the literal, and the next input char is not a space.
                    return (
                        Err(anyhow!(
                            "Next char:'{}' was not whitespace or 'end of str' after literal.",
                            c
                        )),
                        None,
                    );
                }
                (Some(_), None) => {
                    return (
                        Err(anyhow!(
                            "The input: \"{}\" is shorter then the literal: \"{}\"",
                            &input,
                            self.value.as_str()
                        )),
                        None,
                    );
                }
                (Some(lit_c), Some(inp_c)) => {
                    if lit_c == inp_c {
                        ofsett += lit_c.len_utf8();
                        continue;
                    } else {
                        return (Err(anyhow!("Literal did not match input")), None);
                    }
                }
            }
        }
    }

    fn regex(&self) -> String {
        regex_syntax::escape(self.value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::evaluator::Evaluator;

    use super::*;

    #[test]
    fn simple() {
        let lit = Literal {
            value: String::from("tp"),
        };

        let input = &mut "tp 10 10 10";

        let eval = Evaluator::new(&lit);

        let res = eval.evaluate_all(input);

        //println!("{:?}", res);
        assert!(res.iter().any(|x| x.is_ok()));
    }

    #[test]
    fn empty() {
        let lit = Literal {
            value: String::from("tp"),
        };

        let input = "";

        let eval = Evaluator::new(&lit);

        let res = eval.evaluate_all(input);

        assert!(res.iter().all(|x| x.is_err()));
    }

    #[test]
    fn partial() {
        let lit = Literal {
            value: String::from("tp"),
        };

        let input = "tpme";

        let eval = Evaluator::new(&lit);

        let res = eval.evaluate_all(input);

        assert!(res.len() == 1);
        assert!(res[0].is_err());
    }

    #[test]
    fn case() {
        let lit = Literal {
            value: String::from("tp"),
        };

        let input = "tp me";

        let eval = Evaluator::new(&lit);
        let res = eval.evaluate_all(input);
        assert!(res.len() == 1);
        assert!(res.get(0).unwrap().as_ref().unwrap().1 == " me");
    }
}
