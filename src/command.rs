use crate::{ArgumentChecker, Context};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::future::Future;
use std::pin::Pin;

pub trait Command<C: Context> {
    /// Returns the root node for parsing this command.
    fn build(self) -> CommandSpec<C>;
}

pub enum Argument<C: Context> {
    Literal {
        value: Cow<'static, str>,
    },
    Parser {
        name: Cow<'static, str>,
        checker: Box<dyn ArgumentChecker<C>>,
        priority: usize,
    },
}

impl<C: Context> Argument<C> {
    pub fn priority(&self) -> usize {
        match self {
            Argument::Literal { .. } => 0,
            Argument::Parser { priority, .. } => *priority,
        }
    }
}

impl<C: Context> Clone for Argument<C> {
    fn clone(&self) -> Self {
        match self {
            Argument::Literal { value } => Argument::Literal {
                value: value.clone(),
            },
            Argument::Parser {
                name,
                checker,
                priority,
            } => Argument::Parser {
                name: name.clone(),
                checker: checker.box_clone(),
                priority: *priority,
            },
        }
    }
}

impl<C: Context> PartialEq for Argument<C>
where
    C: Context,
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

impl<C: Context> Eq for Argument<C> where C: 'static {}

impl<C: Context> PartialOrd for Argument<C>
where
    C: 'static,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<C: Context> Ord for Argument<C>
where
    C: 'static,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority().cmp(&other.priority())
    }
}

pub type Exec<C> = for<'a> fn(
    &'a mut C,
    &'a str,
) -> Pin<
    Box<dyn Future<Output = Result<<C as Context>::Ok, <C as Context>::Err>> + Send + 'a>,
>;

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
