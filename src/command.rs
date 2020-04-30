use crate::{ArgumentChecker, Context};
use std::borrow::Cow;

pub type Exec<C> = fn(&mut C, &str) -> Result<<C as Context>::Ok, <C as Context>::Error>;

pub trait Command<C: Context> {
    /// Returns the root node for parsing this command.
    fn build(self) -> CommandSpec<C>;
}

pub enum Argument<C> {
    Literal {
        value: Cow<'static, str>,
    },
    Parser {
        name: Cow<'static, str>,
        checker: Box<dyn ArgumentChecker<C>>,
    },
}

impl<C> Clone for Argument<C> {
    fn clone(&self) -> Self {
        match self {
            Argument::Literal { value } => Argument::Literal {
                value: value.clone(),
            },
            Argument::Parser { name, checker } => Argument::Parser {
                name: name.clone(),
                checker: checker.box_clone(),
            },
        }
    }
}

impl<C> PartialEq for Argument<C>
where
    C: 'static,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Argument::Literal { value }, Argument::Literal { value: other }) => value == other,
            (Argument::Parser { checker, .. }, Argument::Parser { checker: other, .. }) => {
                checker.equals(other)
            }
            (_, _) => false,
        }
    }
}

pub struct CommandSpec<C: Context> {
    pub arguments: Vec<Argument<C>>,
    pub description: Option<Cow<'static, str>>,
    pub exec: Exec<C>,
}

impl<C: Context> Command<C> for CommandSpec<C> {
    fn build(self) -> CommandSpec<C> {
        self
    }
}
