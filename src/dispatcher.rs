use crate::parser::{Parser, ParserBase, Result};
use crate::parsers::{Literals, literals};
use crate::{Command, State};
use std::borrow::Cow;

pub struct CommandDispatcher<E> {
    literals: Literals<E>,
}

impl<'a, E> Default for CommandDispatcher<E> {
    fn default() -> Self {
        CommandDispatcher {
            literals: Default::default(),
        }
    }
}

impl<E> CommandDispatcher<(E,)>
where
    E: Command,
{
    pub fn new() -> Self {
        Self {
            literals: Default::default(),
        }
    }

    pub fn call<S>(&self, state: S, command: &str) -> Result<E::Output>
    where
        S: State,
    {
        Ok(self.literals.parse(&mut command.into())?.0.call(state))
    }

    pub fn register<C, L>(&mut self, lit: L, command: C)
    where
        C: 'static + Parser<Extract = (E,)>,
        L: Into<Cow<'static, str>>,
    {
        self.literals.insert(lit, command.boxed());
    }
}