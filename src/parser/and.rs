use crate::generic::{Combine, CombinedTuples, Tuple};

use super::parser::IterParser;



pub struct And<A, B> {
    pub(crate) a: A,
    pub(crate) b: B,
}

pub struct AndState<A: IterParser, B: IterParser> {
    pub(crate) a_state: Option<A::ParserState>,
    pub(crate) a_ext: Option<(A::Extract, usize)>,
    pub(crate) b_state: Option<B::ParserState>,
}

impl<A, B> Default for AndState<A, B>
where
    A: IterParser,
    B: IterParser,
{
    fn default() -> Self {
        Self {
            a_state: Some(A::ParserState::default()),
            b_state: None,
            a_ext: None,
        }
    }
}

impl<A, B> std::fmt::Debug for AndState<A, B>
where
    A: IterParser,
    A::Extract: std::fmt::Debug,
    A::ParserState: std::fmt::Debug,
    B: IterParser,
    B::Extract: std::fmt::Debug,
    B::ParserState: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // match (self.a_state, self.b_state, self.a_ext) {
        //     (None, None, None) => {}
        //     (None, None, Some(_)) => {}
        //     (None, Some(_), None) => {}
        //     (None, Some(_), Some(_)) => {}
        //     (Some(_), None, None) => {}
        //     (Some(_), None, Some(_)) => {}
        //     (Some(_), Some(_), None) => {}
        //     (Some(_), Some(_), Some(_)) => {}
        // }
        match &self.a_ext {
            Some((_ext, pos)) => write!(
                f,
                "(AndState: a_state: {:?}, b_state: {:?}, a_ext: Some((ext:???, pos:{})) )",
                self.a_state, self.b_state, pos
            ),
            None => write!(
                f,
                "(AndState: a_state: {:?}, b_state: {:?}, a_ext: None )",
                self.a_state, self.b_state
            ),
        }
    }
}

impl<A, B> IterParser for And<A, B>
where
    A: IterParser,
    A::Extract: Clone, // I would love some help trying to remove this clone.
    B: IterParser,
    <<A as IterParser>::Extract as Tuple>::HList:
        Combine<<<B as IterParser>::Extract as Tuple>::HList>,
{
    type Extract = CombinedTuples<A::Extract, B::Extract>;
    type ParserState = AndState<A, B>;
    
    #[allow(clippy::type_complexity,clippy::needless_return)]
    fn parse<'p>(
        &self,
        state: AndState<A, B>,
        input: &'p str,
    ) -> (
        anyhow::Result<(Self::Extract, &'p str)>,
        Option<AndState<A, B>>,
    ) {
        let a_state = state.a_state;
        let b_state = state.b_state;
        let a_ext = state.a_ext;

        match (a_state, b_state, a_ext) {
            (None, None, None) => unreachable!("None, None, None"),
            (None, None, Some(_)) => unreachable!("None, None, Some"),
            (None, Some(_), None) => unreachable!("None, Some, None"),
            (None, Some(b_state), Some((a_ext, a_out_index))) => {
                // This state is reachable from the intial state (Some, None, None), and (Some, Some, None)

                let (_, a_out) = input.split_at(a_out_index);

                let (b_ext, b_state) = self.b.parse(b_state, a_out);

                match b_ext {
                    Ok((b_ext, b_out)) => {
                        // We had a complete match we can return.

                        match b_state {
                            Some(b_state) => {
                                // The B parser has more to give. This leads us back to this match arm on the next call.

                                let a_out_index = a_out.len();

                                let cloned_a_ext = a_ext.clone();
                                (
                                    Ok((a_ext.combine(b_ext), b_out)),
                                    Some(AndState {
                                        a_state: None,
                                        a_ext: Some((cloned_a_ext, a_out_index)),
                                        b_state: Some(b_state),
                                    }),
                                )
                            }
                            None => {
                                // The B parser has no more to give, and in this match arm the A parser also has nothing more
                                // to give, resulting in this beeing final match for the And.
                                return (Ok((a_ext.combine(b_ext), b_out)), None);
                            }
                        }
                    }
                    Err(err) => {
                        // We did not have a match

                        match b_state {
                            Some(b_state) => {
                                // More possible matches from the B parser. This leads us back to this match branch on the next call to parse.
                                let a_out_index = a_out.len();
                                return (
                                    Err(err),
                                    Some(AndState {
                                        a_state: None,
                                        a_ext: Some((a_ext, a_out_index)),
                                        b_state: Some(b_state),
                                    }),
                                );
                            }
                            None => {
                                // No more matches comming from B parser, and since we in this branch dont expect any more from A, we know there wont
                                // be any more matches.
                                return (Err(err), None);
                            }
                        }
                    }
                }
            }
            (Some(a_state), None, None) => {
                // This is the entry point when calling parse with a fresh instance of Self::State::default(),
                // This is also reachable from (Some, None, Some)

                let (a_ext, a_state) = self.a.parse(a_state, input);

                match a_ext {
                    Err(err) => {
                        match a_state {
                            None => {
                                // A has no more possibilities, and we did not get a match, so we return the error, and signal that
                                // the and has no possible matches.
                                return (Err(err), None);
                            }
                            Some(a_state) => {
                                // We did not have a match but we have more attempts for the A parser available, this leads us back to this
                                // match arm on the next call.
                                return (
                                    Err(err),
                                    Some(AndState {
                                        a_state: Some(a_state),
                                        a_ext: None,
                                        b_state: None,
                                    }),
                                );
                            }
                        }
                    }
                    Ok((a_ext, a_out)) => {
                        // We have had our first match here for the a parser, now we need to start looking for b matches.
                        match a_state {
                            Some(a_state) => {
                                // This leads us into  (Some, Some, Some)
                                let a_out_index = a_out.len();
                                return self.parse(
                                    AndState {
                                        a_state: Some(a_state),
                                        a_ext: Some((a_ext, a_out_index)),
                                        b_state: Some(B::ParserState::default()),
                                    },
                                    input,
                                );
                            }
                            None => {
                                // No more inputs are possible for A, leading us into (None, Some,Some)

                                let a_out_index = input.len() - a_out.len();
                                return self.parse(
                                    AndState {
                                        a_state: None,
                                        a_ext: Some((a_ext, a_out_index)),
                                        b_state: Some(B::ParserState::default()),
                                    },
                                    input,
                                );
                            }
                        }
                    }
                }
            }
            (Some(a_state), None, Some(_a_ext)) => {
                // This branch is reachable from (Some, Some, Some)

                // We had a match and no more matches are comming for B.
                // This means that we should parse a again,thereby reset/update the a_ext,
                // and finally reset b_state to default.

                // Note: this is the branch you are workin on.
                //let a_state_clone = a_state.clone();

                return self.parse(
                    AndState {
                        a_state: Some(a_state),
                        a_ext: None,
                        b_state: None,
                    },
                    input,
                );

                //unreachable!("Some, None, Some")
            }
            (Some(a_state), Some(b_state), None) => {
                // We can get in this branch from the (Some, Some, Some). This state happens when for a given input have
                // had a match for A, and then gone through all possible states for B. We now need to update our attempt
                // at parsing with the first parser A.

                let (a_ext, a_state) = self.a.parse(a_state, input);

                match (a_ext, a_state) {
                    (Ok((a_ext, a_out)), None) => {
                        // We got a new match and no more new ones are comming for a parser.
                        // this leads us to (None,Some,Some)
                        let a_out_index = a_out.len();
                        return self.parse(
                            AndState {
                                a_state: None,
                                a_ext: Some((a_ext, a_out_index)),
                                b_state: Some(b_state),
                            },
                            input,
                        );
                    }
                    (Ok((a_ext, a_out)), Some(a_state)) => {
                        // This leads us into the Some, Some, Some branch
                        let a_out_index = a_out.len();
                        return self.parse(
                            AndState {
                                a_state: Some(a_state),
                                a_ext: Some((a_ext, a_out_index)),
                                b_state: Some(b_state),
                            },
                            input,
                        );
                    }
                    (Err(err), None) => {
                        // We had no match, and no more matches possible for the a parser.
                        return (Err(err), None);
                    }
                    (Err(err), Some(a_state)) => {
                        // We had no match but more matches are possible for the a parser.
                        // This leads us back to this branch in the next call to parse.
                        return (
                            Err(err),
                            Some(AndState {
                                a_state: Some(a_state),
                                a_ext: None,
                                b_state: Some(b_state),
                            }),
                        );
                    }
                }
            }
            (Some(a_state), Some(b_state), Some((a_ext, a_out_index))) => {
                // This state is reachable from the entrypoint (Some,None,None)
                // and also (Some,Some,None)

                // We continue trying different cases of the b_parser.
                let (_, a_out) = input.split_at(a_out_index);
                let (b_ext, b_state) = self.b.parse(b_state, a_out);

                match (b_ext, b_state) {
                    (Ok((b_ext, b_out)), None) => {
                        // We had a match and no more matches are comming for B.
                        // This means that on the next call we should parse a again,thereby
                        // reset/update the a_ext, and finally reset b_state to default.

                        // This results in the (Some, None, Some) on the next call.
                        let cloned_a_ext = a_ext.clone();
                        return (
                            Ok((a_ext.combine(b_ext), b_out)),
                            Some(AndState {
                                a_state: Some(a_state),
                                a_ext: Some((cloned_a_ext, a_out_index)),
                                b_state: None,
                            }),
                        );
                    }
                    (Ok((b_ext, b_out)), Some(b_state)) => {
                        // We had a match and there are more potential matches for B.
                        // This results in (Some, Some, Some), returning to this branch on the next call.
                        let cloned_a_ext = a_ext.clone();
                        return (
                            Ok((a_ext.combine(b_ext), b_out)),
                            Some(AndState {
                                a_state: Some(a_state),
                                a_ext: Some((cloned_a_ext, a_out_index)),
                                b_state: Some(b_state),
                            }),
                        );
                    }
                    (Err(err), None) => {
                        // We found nothing and there are no more to get for the B parser.
                        // We therefor reset the B state, and remove (a_ext, a_out).
                        // This moves us into the (Some, Some, None) branch
                        return (
                            Err(err),
                            Some(AndState {
                                a_state: Some(a_state),
                                a_ext: None,
                                b_state: Some(B::ParserState::default()),
                            }),
                        );
                    }
                    (Err(err), Some(b_state)) => {
                        // We found nothing, but there are more to get from the B parser
                        // This sends us back to this branch on the next call to parse.
                        return (
                            Err(err),
                            Some(AndState {
                                a_state: Some(a_state),
                                a_ext: Some((a_ext, a_out_index)),
                                b_state: Some(b_state),
                            }),
                        );
                    }
                }
            }
        }
    }

    fn regex(&self) -> String {
        format!("({})({})", &self.a.regex(), &self.b.regex())
    }
}

mod tests {

    #[test]
    fn simple() {
        let lit1 = crate::parser::Literal {
            value: String::from("tp"),
        };

        let lit2 = crate::parser::Literal {
            value: String::from("me"),
        };

        let and = crate::parser::And { a: lit1, b: lit2 };

        let input = "tp me";

        let eval = crate::parser::Evaluator::new(&and);

        let res = eval.evaluate_all(input);

        assert!(res.len() == 1);
        assert!(res.get(0).unwrap().is_ok());
        assert!(res.len() == 1);
    }

    #[test]
    fn simple_opt_1() {
        let and = crate::parser::And {
            a: crate::parser::Opt {
                parser: crate::parser::Literal {
                    value: String::from("tp"),
                },
            },
            b: crate::parser::Literal {
                value: String::from("me"),
            },
        };

        let input = &mut "tp me";
        let eval = crate::parser::Evaluator::new(&and);
        let res = eval.evaluate_all(input);

        assert!(res.len() == 2);
        assert!(res[0].is_ok());
        assert!(res[1].is_err());
    }

    #[test]
    fn simple_opt_2() {
        let lit1 = crate::parser::Literal {
            value: String::from("tp"),
        };

        let opt1 = crate::parser::Opt { parser: lit1 };

        let lit2 = crate::parser::Literal {
            value: String::from("me"),
        };

        let and = crate::parser::And { a: opt1, b: lit2 };

        let input = "me ";

        let eval = crate::parser::Evaluator::new(&and);

        let res = eval.evaluate_all(input);

        assert!(res.len() == 2);
        assert!(res[0].is_err());
        assert!(res[1].is_ok());
    }

    #[test]
    fn simple_opt_3() {
        let lit1 = crate::parser::Literal {
            value: String::from("tp"),
        };

        let opt1 = crate::parser::Opt { parser: lit1 };

        let lit2 = crate::parser::Literal {
            value: String::from("me"),
        };

        let opt2 = crate::parser::Opt { parser: lit2 };

        let and = crate::parser::And { a: opt1, b: opt2 };

        let input = " ";

        let eval = crate::parser::Evaluator::new(&and);

        let res = eval.evaluate_all(input);

        assert!(res.len() == 3);
        assert!(res[0].is_err());
        assert!(res[1].is_err());
        assert!(res[2].is_ok());
    }

    #[test]
    fn simple_opt_4() {
        for word in vec!["tp", "tango", "121", "œeœ", "ࢰࢰ", "😈😈😈"] {
            let lit1 = crate::parser::Literal {
                value: String::from(word),
            };

            let opt1 = crate::parser::Opt { parser: lit1 };

            let lit2 = crate::parser::Literal {
                value: String::from(word),
            };

            let opt2 = crate::parser::Opt { parser: lit2 };

            let and = crate::parser::And { a: opt1, b: opt2 };

            let input = format!("{} {}", word, word);

            let eval = crate::parser::Evaluator::new(&and);

            let res = eval.evaluate_all(input.as_str());

            //println!("{:?}",res);
            assert!(res.len() == 4);
            assert!(res.iter().all(|x| x.is_ok()));
        }
    }
}
