use crate::ArgumentChecker;
use std::borrow::Cow;

pub trait Command<C> {
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

pub struct CommandSpec<C> {
    pub arguments: Vec<Argument<C>>,
    pub description: Option<Cow<'static, str>>,
    pub exec: Box<fn(&mut C, &str)>,
}
