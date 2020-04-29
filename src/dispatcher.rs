use crate::{ArgumentChecker, Command, CommandNode, CommandNodeKind};
use slab::Slab;
use smallvec::SmallVec;
use std::borrow::Cow;

#[derive(Debug)]
pub enum RegisterError {
    /// Overlapping commands exist: two commands
    /// have an executable node at the same point.
    OverlappingCommands,
    /// Attempted to register an executable command at the root of the command graph.
    ExecutableRoot,
}

#[derive(Copy, Clone, Debug)]
struct NodeKey(usize);

/// Data structure used to dispatch commands.
pub struct CommandDispatcher {
    nodes: Slab<Node>,
    root: NodeKey,
}

impl Default for CommandDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandDispatcher {
    /// Creates a new `CommandDispatcher` with no registered commands.
    pub fn new() -> Self {
        let mut nodes = Slab::new();
        let root = NodeKey(nodes.insert(Node::default()));

        Self { nodes, root }
    }

    /// Registers a command to this `CommandDispatcher`.
    pub fn register(&mut self, command: impl Command) -> Result<(), RegisterError> {
        self.append_node(self.root, command.into_root_node())
    }

    /// Method-chaining function to register a command.
    ///
    /// # Panics
    /// Panics if overlapping commands are detected. Use `register`
    /// to handle this error.
    pub fn with(mut self, command: impl Command) -> Self {
        self.register(command).unwrap();
        self
    }

    /// Dispatches a command. Returns whether a command was executed.
    ///
    /// Unicode characters are currently not supported. This may be fixed in the future.
    pub fn dispatch(&self, command: &str) -> bool {
        let parsed = Self::parse_into_arguments(command);

        let mut current_node = self.root;

        for argument in &parsed {
            // try to find a node satisfying the argument
            let node = &self.nodes[current_node.0];

            // TODO: optimize linear search using a hash-array mapped trie
            if let Some(next) = node.next.iter().find(|next| {
                let kind = &self.nodes[next.0].kind;

                match kind {
                    NodeKind::Parser(parser) => parser.satisfies(argument),
                    NodeKind::Literal(lit) => lit == argument,
                    NodeKind::Root => unreachable!("root NodeKind outside the root node?"),
                }
            }) {
                current_node = *next;
            } else {
                return false;
            }
        }

        if let Some(exec) = &self.nodes[current_node.0].exec {
            exec(&parsed);
            true
        } else {
            false
        }
    }

    fn parse_into_arguments(command: &str) -> SmallVec<[&str; 4]> {
        // TODO: proper parser with support for strings in quotes
        command.split(" ").collect()
    }

    fn append_node(
        &mut self,
        dispatcher_current: NodeKey,
        cmd_current: CommandNode,
    ) -> Result<(), RegisterError> {
        if let Some(exec) = cmd_current.exec {
            let node = &mut self.nodes[dispatcher_current.0];

            if let NodeKind::Root = node.kind {
                return Err(RegisterError::ExecutableRoot);
            }

            match node.exec {
                Some(_) => return Err(RegisterError::OverlappingCommands),
                None => node.exec = Some(exec),
            }
        }

        let cmd_current_kind = &cmd_current.kind;

        // Find a node which has the same parser type as `cmd_current`,
        // or add it if it doesn't exist.
        let found = self.nodes[dispatcher_current.0]
            .next
            .iter()
            .find(|key| &self.nodes[key.0].kind == cmd_current_kind)
            .copied();

        let found = if let Some(found) = found {
            found
        } else {
            // Create new node, then append.
            let new_node = self.nodes.insert(Node::from(cmd_current.kind));

            self.nodes[dispatcher_current.0]
                .next
                .push(NodeKey(new_node));

            NodeKey(new_node)
        };
        cmd_current
            .next
            .into_iter()
            .map(|next| self.append_node(found, next))
            .collect::<Result<(), RegisterError>>()?;

        Ok(())
    }
}

/// Node on the command graph.
#[derive(Default)]
struct Node {
    next: SmallVec<[NodeKey; 4]>,
    kind: NodeKind,
    exec: Option<Box<dyn Fn(&[&str])>>,
}

impl From<CommandNodeKind> for Node {
    fn from(node: CommandNodeKind) -> Self {
        Node {
            next: SmallVec::new(),
            kind: match node {
                CommandNodeKind::Literal(lit) => NodeKind::Literal(lit),
                CommandNodeKind::Parser(parser) => NodeKind::Parser(parser),
            },
            exec: None,
        }
    }
}

enum NodeKind {
    Literal(Cow<'static, str>),
    Parser(Box<dyn ArgumentChecker>),
    Root,
}

impl PartialEq<CommandNodeKind> for NodeKind {
    fn eq(&self, other: &CommandNodeKind) -> bool {
        match (self, other) {
            (NodeKind::Literal(this), CommandNodeKind::Literal(other)) => this.eq(other),
            (NodeKind::Parser(this), CommandNodeKind::Parser(other)) => this.equals(other),
            _ => false,
        }
    }
}

impl Default for NodeKind {
    fn default() -> Self {
        NodeKind::Root
    }
}

#[cfg(test)]
mod tests {
    /*use super::*;
    use bstr::B;
    use smallvec::smallvec;

    #[test]
    fn parse_into_arguments() {
        let test: Vec<(&[u8], SmallVec<[&[u8]; 4]>)> = vec![
            (
                B("test 20 \"this is a string: \\\"Hello world\\\"\""),
                smallvec![B("test"), B("20"), B("this is a string: \"Hello world\"")],
            ),
            (
                B("big inputs cost big programmers with big skills"),
                smallvec![
                    B("big"),
                    B("inputs"),
                    B("cost"),
                    B("big"),
                    B("programmers"),
                    B("with"),
                    B("big"),
                    B("skills"),
                ],
            ),
        ];

        for (input, expected) in test {
            assert_eq!(CommandDispatcher::parse_into_arguments(input), expected);
        }
    }*/
}
