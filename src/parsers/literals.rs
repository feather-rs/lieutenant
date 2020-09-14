use crate::parser::{Parser, ParserBase, Tuple, Result, Error};
use crate::Input;
use std::collections::HashMap;
use unicase::UniCase;
use std::borrow::Cow;

pub struct Literals<E>(HashMap<UniCase<Cow<'static, str>>, Box<dyn Parser<Extract = E>>>);

impl<E> Literals<E> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<L>(&mut self, lit: L, parser: Box<dyn Parser<Extract = E>>)
    where
        L: Into<Cow<'static, str>>,
    {
        let lit = lit.into();
        assert!(!lit.is_empty());
        assert!(lit.chars().all(|c| c != ' '));
        self.0.insert(UniCase::new(lit), parser);
    }
}

impl<'a, E> Default for Literals<E> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<'a, E> ParserBase for Literals<E>
where
    E: Tuple,
{
    type Extract = E;

    fn parse<'i>(&self, input: &mut Input<'i>) -> Result<Self::Extract> {
        let head = input.advance_until(" ");
        let head = UniCase::new(head.into());
        self.0.get(&head).ok_or(Error::Literals(vec![]))?.parse(input)
    }
}

pub fn literals<'a, P, L, LS>(literals: LS) -> Literals<P::Extract>
where 
    P: 'static + Parser,
    L: Into<Cow<'static, str>>,
    LS: IntoIterator<Item = (L, P)>,
{
    Literals(literals.into_iter().map(|(lit, parser)| (UniCase::new(lit.into()), parser.boxed())).collect())
}
