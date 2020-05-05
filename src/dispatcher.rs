use crate::{command::Exec, Argument, Command, CommandSpec, Context, Input};
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NodeKey(usize);

impl std::ops::Deref for NodeKey {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// State of an ongoing dispatch request.
///
/// Dispatching is driven by the library user.
/// When a command needs to be dispatched,
/// call `CommandDispatcher::begin(command)`
/// to get a `DispatchState`. Then, call
/// `CommandDispatcher::drive(state)` until a command
/// succeeds.
pub struct DispatchState<'a> {
    /// The entirety of the command being dispatched.
    pub command: Cow<'a, str>,
    /// The node stack, used for depth-first search.
    ///
    /// The first element of the tuple is the index
    /// into `command` at which the remainder of the
    /// input being parsed begins. The second
    /// element is the node being handled.
    nodes: Vec<(usize, NodeKey)>,
}

/// Returned by `CommandDispatcher::drive`.
pub enum DriveResult<C: Context> {
    /// Encountered an executable node
    /// which may be executed. It is up
    /// to the user to handle the actual
    /// execution of the command.
    Exec(Exec<C>),
    /// No more commands to search.
    Finished,
}

/// Data structure used to dispatch commands.
pub struct CommandDispatcher<C: Context> {
    // This structure acts as the root node.
    /// Stores all nodes in the command graph.
    nodes: Slab<Node<C>>,
    /// Children of the root node.
    children: SmallVec<[NodeKey; 4]>,
    /// Vector of all commands registered to this dispatcher.
    commands: Vec<CommandSpec<C>>,
}

impl<C: Context> Default for CommandDispatcher<C> {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            children: Default::default(),
            commands: Default::default(),
        }
    }
}

impl<C> CommandDispatcher<C>
where
    C: Context,
{
    /// Creates a new `CommandDispatcher` with no registered commands.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a command to this `CommandDispatcher`.
    pub fn register(&mut self, command: impl Command<C>) -> Result<(), RegisterError>
    where
        C: 'static,
        <C as Context>::ArgumentPayload: Clone,
    {
        let spec = command.build();

        let mut arguments = spec.arguments.iter().peekable();

        let mut node_key: Option<NodeKey> = None;

        'argument: while let Some(argument) = arguments.peek() {
            let children = match node_key {
                Some(key) => &self.nodes[*key].children,
                None => &self.children,
            };

            for child_key in children {
                let child = &self.nodes[**child_key];

                if argument == &&child.argument {
                    arguments.next();
                    node_key = Some(*child_key);
                    continue 'argument;
                }
            }
            break;
        }

        for argument in arguments {
            let child = Node::from((*argument).clone());
            let child_key = NodeKey(self.nodes.insert(child));

            if let Some(node_key) = node_key {
                let node = &mut self.nodes[*node_key];
                node.children.push(child_key);
            } else {
                self.children.push(child_key);
            }

            node_key = Some(child_key);
        }

        if let Some(key) = node_key {
            let node = &mut self.nodes[*key];
            node.exec = Some(spec.exec);
        } else {
            // Command with zero arguments?
            return Err(RegisterError::ExecutableRoot);
        }

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
        <C as Context>::ArgumentPayload: Clone,
    {
        self.register(command).unwrap();
        self
    }

    /// Begins dispatching a command by initializing
    /// a `DispatchState`. The user may call `drive`
    /// to continue driving command dispatch.
    pub fn begin<'a>(&self, command: impl Into<Cow<'a, str>>) -> DispatchState<'a> {
        DispatchState {
            command: command.into(),
            nodes: self
                .children
                .iter()
                .copied()
                .map(|child_key| (0, child_key))
                .collect(),
        }
    }

    /// Drives dispatching of a command. Call `drive`
    /// to first obtain a `DispatchState`.
    pub fn drive(&self, state: &mut DispatchState) -> DriveResult<C> {
        while let Some((cursor, node_key)) = state.nodes.pop() {
            let node = &self.nodes[*node_key];
            let mut input = Input::new(&state.command[cursor..]);

            let may_satisfy = match &node.argument {
                Argument::Literal { values } => {
                    let parsed = input.advance_until(" ");
                    values.iter().any(|value| value == parsed)
                }
                Argument::Parser { may_satisfy, .. } => may_satisfy(&mut input),
            };

            if may_satisfy && input.is_empty() {
                // at end of input - try executing
                if let Some(exec) = node.exec {
                    return DriveResult::Exec(exec);
                }
            }

            if may_satisfy {
                for child_key in &node.children {
                    state
                        .nodes
                        .push((state.command.len() - input.len(), *child_key));
                }
            }
        }

        DriveResult::Finished
    }

    pub fn commands(&self) -> impl Iterator<Item = &CommandSpec<C>> {
        self.commands.iter()
    }
}

/// Node on the command graph.
struct Node<C: Context> {
    children: SmallVec<[NodeKey; 4]>,
    argument: Argument<C>,
    exec: Option<Exec<C>>,
}

impl<C: Context> From<Argument<C>> for Node<C> {
    fn from(argument: Argument<C>) -> Self {
        Node {
            children: Default::default(),
            argument,
            exec: None,
        }
    }
}
