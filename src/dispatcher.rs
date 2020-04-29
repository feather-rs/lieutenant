use crate::{Argument, ArgumentChecker, Command, CommandSpec, Input};
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
pub struct CommandDispatcher<C> {
    nodes: Slab<Node<C>>,
    root: NodeKey,
    commands: Vec<CommandSpec<C>>,
}

impl<C> Default for CommandDispatcher<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> CommandDispatcher<C> {
    /// Creates a new `CommandDispatcher` with no registered commands.
    pub fn new() -> Self {
        let mut nodes = Slab::new();
        let root = NodeKey(nodes.insert(Node::default()));
        let commands: Vec<CommandSpec<C>> = Vec::new();

        Self {
            nodes,
            root,
            commands,
        }
    }

    /// Registers a command to this `CommandDispatcher`.
    pub fn register(&mut self, command: impl Command<C>) -> Result<(), RegisterError>
    where
        C: 'static,
    {
        let spec = command.build();

        let mut arguments = spec.arguments.iter();

        let mut current_node_key = &self.root;

        'arguments: while let Some(argument) = arguments.next() {
            let current_node = &self.nodes[current_node_key.0];
            for next_node_key in &current_node.next {
                let next_node = &self.nodes[next_node_key.0];
                match (argument, &next_node.kind) {
                    (Argument::Literal { value }, NodeKind::Literal(node_value))
                        if value == node_value => {
                            current_node_key = next_node_key;
                            continue 'arguments;
                        }
                    (Argument::Parser { checker, .. }, NodeKind::Parser(node_checker))
                        if checker.equals(node_checker) => {
                            current_node_key = next_node_key;
                            continue 'arguments;
                        }
                    (_, NodeKind::Root) => panic!("?"),
                    _ => continue,
                }
            }
        }

        let mut current_node_key = current_node_key.clone();

        while let Some(argument) = arguments.next() {
            let next_node = Node::from(argument.clone());
            let next_node_key = NodeKey(self.nodes.insert(next_node));
            let current_node = &mut self.nodes[current_node_key.0.clone()];
            current_node.next.push(next_node_key.clone());
            current_node_key = next_node_key;
        }

        let mut current_node = &mut self.nodes[current_node_key.0];

        if current_node.exec.is_some() {
            return Err(RegisterError::OverlappingCommands);
        }

        current_node.exec = Some(spec.exec.clone());

        self.commands.push(spec);

        Ok(())
    }

    /// Method-chaining function to register a command.
    ///
    /// # Panics
    /// Panics if overlapping commands are detected. Use `register`
    /// to handle this error.
    pub fn with(mut self, command: impl Command<C>) -> Self
    where
        C: 'static,
    {
        self.register(command).unwrap();
        self
    }

    /// Dispatches a command. Returns whether a command was executed.
    ///
    /// Unicode characters are currently not supported. This may be fixed in the future.
    pub fn dispatch(&self, ctx: &mut C, command: &str) -> bool {
        // let parsed = Self::parse_into_arguments(command);

        let mut current_node = self.root;

        let mut input = Input::new(command);

        while !input.empty() {
            // try to find a node satisfying the argument
            let node = &self.nodes[current_node.0];

            // TODO: optimize linear search using a hash-array mapped trie
            if let Some((next, next_input)) = node
                .next
                .iter()
                .filter_map(|next| {
                    let kind = &self.nodes[next.0].kind;
                    let mut input = input.clone();

                    if match kind {
                        NodeKind::Parser(parser) => parser.satisfies(ctx, &mut input),
                        NodeKind::Literal(lit) => lit == input.head(" "),
                        NodeKind::Root => unreachable!("root NodeKind outside the root node?"),
                    } {
                        Some((next, input))
                    } else {
                        None
                    }
                })
                .next()
            {
                current_node = *next;
                input = next_input;
            } else if let Some(exec) = &self.nodes[current_node.0].exec {
                exec(ctx, command);
                return true;
            } else {
                return false;
            }
        }
        false
    }

    pub fn commands(&self) -> impl Iterator<Item = &CommandSpec<C>> {
        self.commands.iter()
    }
}

/// Node on the command graph.
struct Node<C> {
    next: SmallVec<[NodeKey; 4]>,
    kind: NodeKind<C>,
    exec: Option<Box<dyn Fn(&mut C, &str)>>,
}

impl<C> Default for Node<C> {
    fn default() -> Self {
        Self {
            next: SmallVec::new(),
            kind: NodeKind::<C>::default(),
            exec: None,
        }
    }
}

impl<C> From<Argument<C>> for Node<C> {
    fn from(node: Argument<C>) -> Self {
        Node {
            next: SmallVec::new(),
            kind: match node {
                Argument::Literal { value } => NodeKind::Literal(value),
                Argument::Parser { checker, .. } => NodeKind::Parser(checker),
            },
            exec: None,
        }
    }
}

enum NodeKind<C> {
    Literal(Cow<'static, str>),
    Parser(Box<dyn ArgumentChecker<C>>),
    Root,
}

impl<C> PartialEq<Argument<C>> for NodeKind<C>
where
    C: 'static,
{
    fn eq(&self, other: &Argument<C>) -> bool {
        match (self, other) {
            (NodeKind::Literal(this), Argument::Literal { value: other}) => this.eq(other),
            (NodeKind::Parser(this), Argument::Parser { checker: other, .. }) => this.equals(other),
            _ => false,
        }
    }
}

impl<C> Default for NodeKind<C> {
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
