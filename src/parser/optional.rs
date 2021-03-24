use crate::generic::Tuple;

use super::parser::IterParser;

pub struct Opt<P> {
    pub(crate) parser: P,
}

#[derive(Debug)]
pub enum OptState<State> {
    Consume(State),
    Skip(),
}

impl<State: Default> Default for OptState<State> {
    fn default() -> Self {
        Self::Consume(State::default())
    }
}

impl<P: IterParser> IterParser for Opt<P>
where
    P::Extract: Tuple,
    P::ParserState: Default,
{
    type Extract = (Option<P::Extract>,);
    type ParserState = OptState<P::ParserState>;

    #[allow(clippy::type_complexity)]
    fn parse<'p>(
        &self,
        state: Self::ParserState,
        input: &'p str,
    ) -> (
        anyhow::Result<(Self::Extract, &'p str)>,
        Option<Self::ParserState>,
    ) {
        match state {
            OptState::Skip() => {
                // We have parsed the input consuming, and the sub parser has stopped giving us alternatives.
                // We need to switch tactics and try to not parse anything, returning None.
                (Ok(((None,), input)), None)
            }
            OptState::Consume(sub_parser_state) => {
                // We parse the input trying to consume the start.
                let (res, sub_parser_state) = self.parser.parse(sub_parser_state, input);

                match (res, sub_parser_state) {
                    (Ok((ext, out)), None) => {
                        // Match and no more cases for the sub parser
                        (Ok(((Some(ext),), out)), Some(OptState::Skip()))
                    }
                    (Ok((ext, out)), Some(new_state)) => {
                        (Ok(((Some(ext),), out)), Some(OptState::Consume(new_state)))
                    }
                    (Err(err), None) => {
                        (Err(err), Some(OptState::Skip()))
                    }
                    (Err(err), Some(new_state)) => {
                        (Err(err), Some(OptState::Consume(new_state)))
                    }
                }
            }
        }
    }
    fn regex(&self) -> String {
        format!("({})?", self.parser.regex())
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::evaluator::Evaluator;
    use crate::parser::literal::Literal;

    use super::*;

    #[test]
    fn simple1() {
        let lit = Literal {
            value: String::from("tp"),
        };

        let opt = Opt { parser: lit };

        let input = "tp";

        let eval = Evaluator::new(&opt);

        let res = eval.evaluate_all(input);

        assert!(res.iter().all(|x| x.is_ok()));
        println!("{:?}", res);
    }

    #[test]
    fn simple2() {
        let lit = Literal {
            value: String::from("tp"),
        };

        let opt = Opt { parser: lit };

        let input = "tp me";

        let eval = Evaluator::new(&opt);

        let res = eval.evaluate_all(input);

        //assert!(res.iter().all(|x| x.is_ok()));
        println!("{:?}", res);
    }

    #[test]
    fn empty() {
        let lit = Literal {
            value: String::from("tp"),
        };

        let opt = Opt { parser: lit };

        let input = "";

        let eval = Evaluator::new(&opt);

        let res = eval.evaluate_all(input);
        assert!(res.get(0).unwrap().is_err());
        assert!(res.get(1).unwrap().is_ok());
        assert!(res.len() == 2);
    }

    #[test]
    fn partial() {
        let lit = Literal {
            value: String::from("tp"),
        };

        let input = "tpme";

        let eval = Evaluator::new(&lit);

        let res = eval.evaluate_all(input);

        assert!(res.iter().all(|x| x.is_err()));
    }
}
