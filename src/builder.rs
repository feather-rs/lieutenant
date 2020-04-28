use crate::cons::{ConsAppend, ConsFlatten};
use crate::{ArgumentKind, ArgumentParser, Command, CommandNode, CommandNodeKind};
use std::borrow::Cow;
use std::marker::PhantomData;

pub struct CommandBuilder<ARGS> {
    _args: PhantomData<ARGS>,
    root: CommandNode,
}

impl CommandBuilder<()> {
    /// Creates a new `CommandBuilder` to build
    /// a command. The first node of the command
    /// will be a literal matching `name`.
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            _args: Default::default(),
            root: CommandNode {
                kind: CommandNodeKind::Literal(name.into()),
                next: vec![],
                exec: None,
            },
        }
    }
}

impl<ARGS> CommandBuilder<ARGS> {
    /// Appends a literal matcher to the node sequence.
    pub fn literal(mut self, lit: impl Into<Cow<'static, str>>) -> Self {
        self.append(CommandNodeKind::Literal(lit.into()));

        self
    }

    /// Appends a parsed argument matcher to the node sequence.
    pub fn arg<T>(mut self) -> CommandBuilder<<ARGS as ConsAppend<T>>::Output>
    where
        T: ArgumentKind,
        <T as ArgumentKind>::Checker: Default,
        ARGS: ConsAppend<T>,
    {
        self.append(CommandNodeKind::Parser(Box::new(T::Checker::default())));

        CommandBuilder::<<ARGS as ConsAppend<T>>::Output> {
            _args: Default::default(),
            root: self.root,
        }
    }

    fn append(&mut self, kind: CommandNodeKind) {
        let node = CommandNode {
            kind,
            next: vec![],
            exec: None,
        };
        self.append_node(node);
    }

    fn append_node(&mut self, node: CommandNode) {
        let mut prev = &mut self.root;

        loop {
            if !prev.next.is_empty() {
                prev = prev.next.first_mut().unwrap();
            } else {
                prev.next.push(node);
                break;
            }
        }
    }
}

impl<ARGS> CommandBuilder<ARGS>
where
    ARGS: ConsFlatten,
    <ARGS as ConsFlatten>::Output: TupleParse,
{
    /// Finishes building a command, returning a `Command` instance
    /// to add to a `CommandDispatcher`.
    pub fn build(mut self, exec: impl Fn(<ARGS as ConsFlatten>::Output) + 'static) -> BuiltCommand {
        self.append_node(CommandNode {
            kind: CommandNodeKind::Literal("".into()), // Will be ignored because this is the final node
            next: vec![],
            exec: Some(Box::new(move |input| {
                let args = <<ARGS as ConsFlatten>::Output as TupleParse>::parse(input).unwrap();
                exec(args);
            })),
        });

        BuiltCommand { root: self.root }
    }
}

/// Output from `CommandBuilder::build()`.
pub struct BuiltCommand {
    root: CommandNode,
}

impl Command for BuiltCommand {
    fn into_root_node(self) -> CommandNode {
        self.root
    }
}

pub trait TupleParse {
    fn parse(command: &[&str]) -> anyhow::Result<Self>
    where
        Self: Sized;
}

macro_rules! impl_parse {
    ($head:ident, $($ty:ident),*) => {
        impl <$head, $($ty),*> TupleParse for ($head, $($ty),*) where $($ty: ArgumentKind),*, $head: ArgumentKind {
            fn parse(command: &[&str]) -> anyhow::Result<Self> where Self: Sized {
                let mut iter = command.iter().skip(1); // skip first argument, which is the command name / literal
                Ok((<$head as ArgumentKind>::Parser::default().parse(*iter.next().unwrap())?, $(
                    <$ty as ArgumentKind>::Parser::default().parse(*iter.next().unwrap())?
                ),*))
            }
        }

        impl_parse!($($ty),*);
    };
    ($head:ident) => {}
}

impl_parse!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

impl<T> TupleParse for T
where
    T: ArgumentKind,
{
    fn parse(command: &[&str]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        <T as ArgumentKind>::Parser::default().parse(command[1])
    }
}
