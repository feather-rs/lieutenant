use crate::ArgumentChecker;
use std::borrow::Cow;

pub struct CommandNode {
    pub kind: CommandNodeKind,
    pub next: Vec<CommandNode>,
    pub exec: Option<Box<dyn Fn(&[&str])>>,
}

pub enum CommandNodeKind {
    Literal(Cow<'static, str>),
    Parser(Box<dyn ArgumentChecker>),
}

pub trait Command {
    /// Returns the root node for parsing this command.
    fn into_root_node(self) -> CommandNode;
}
