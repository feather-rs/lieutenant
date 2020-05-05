use crate::parser::MaySatisfyFn;
use crate::Context;
use derivative::Derivative;
use smallvec::SmallVec;
use std::any::TypeId;
use std::borrow::Cow;
use std::cmp::Ordering;

/// A type which can be converted into a `CommandSpec`.
///
/// This is automatically implemented for functions annotated
/// with the `command` proc macro.
pub trait Command<C: Context> {
    /// Returns the spec of this command, which specifies
    /// its arguments and executable function.
    fn build(self) -> CommandSpec<C>;
}

/// An argument to a command.
#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
pub enum Argument<C: Context> {
    /// A literal, matching some fixed string value.
    ///
    /// Multiple strings may match the same literal
    /// argument, to allow for aliasing.
    Literal {
        /// The set of strings which match this literal.
        values: SmallVec<[Cow<'static, str>; 2]>,
    },
    /// A custom-parsed argument.
    Parser {
        /// Name of this argument.
        name: Cow<'static, str>,
        /// Priority of this argument. Greater priorities
        /// take precedence over command nodes with lower
        /// priorities.
        priority: usize,
        /// The function used to check whether
        /// a given input matches this parser.
        satisfies: MaySatisfyFn<C>,
        /// Type ID of the argument type.
        argument_type: TypeId,
    },
}

impl<C: Context> Argument<C> {
    /// Returns the priority of this argument node.
    pub fn priority(&self) -> usize {
        match self {
            Argument::Literal { .. } => 0,
            Argument::Parser { priority, .. } => *priority,
        }
    }
}

impl<C> PartialEq for Argument<C>
where
    C: Context,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Argument::Literal { values: v1 }, Argument::Literal { values: v2 }) => v1 == v2,
            (
                Argument::Parser {
                    argument_type: s1, ..
                },
                Argument::Parser {
                    argument_type: a2, ..
                },
            ) => s1 == a2,
            _ => false,
        }
    }
}

impl<C> Eq for Argument<C> where C: Context {}

impl<C: Context> PartialOrd for Argument<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<C: Context> Ord for Argument<C> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority().cmp(&other.priority())
    }
}

pub type Exec<C> = fn(&mut C, &str) -> Result<<C as Context>::Ok, <C as Context>::Error>;

/// Specifies the arguments to a command,
/// plus its metadata and executable function.
pub struct CommandSpec<C: Context> {
    /// Argument nodes to this command. This is a list,
    /// not a graph.
    pub arguments: Vec<Argument<C>>,
    /// Description of this command, potentially nonexistent.
    pub description: Option<Cow<'static, str>>,
    /// THe function used to execute this command.
    pub exec: Exec<C>,
}

impl<C: Context> Command<C> for CommandSpec<C> {
    fn build(self) -> CommandSpec<C> {
        self
    }
}
