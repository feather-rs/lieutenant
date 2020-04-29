use crate::ArgumentChecker;
use std::borrow::Cow;

pub struct CommandNode<C> {
    pub kind: CommandNodeKind<C>,
    pub next: Vec<CommandNode<C>>,
    pub exec: Option<Box<dyn Fn(&mut C, &str)>>,
}

pub enum CommandNodeKind<C> {
    Literal(Cow<'static, str>),
    Parser(Box<dyn ArgumentChecker<C>>),
}

pub trait Command<C> {
    /// Returns the root node for parsing this command.
    fn into_root_node(self) -> CommandNode<C>;

    /// Returns the metadata of the node
    fn meta(&self) -> CommandMeta;
}

pub struct CommandMeta {
    pub usage: Cow<'static, str>,
    pub description: Option<Cow<'static, str>>,
}