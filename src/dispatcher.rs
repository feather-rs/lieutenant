use crate::{Error, Result, SyntaxError};
use std::{
    collections::HashMap,
    ops::{Index, IndexMut, RangeInclusive},
};

/// Unique ID of a node within a command dispatcher. Remains stable.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

impl<Ctx> Index<NodeId> for Vec<Node<Ctx>> {
    type Output = Node<Ctx>;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self[index.0]
    }
}

impl<Ctx> IndexMut<NodeId> for Vec<Node<Ctx>> {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self[index.0]
    }
}

/// A command dispatcher.
///
/// This dispatcher maintains a directed graph where
/// each node represents an argument to a command.
///
/// # Algorithm
/// The dispatcher internally keeps a data structure very
/// similar to the one found at https://wiki.vg/Command_Data#Parsers.
/// It keeps a graph of nodes, each of which has some parameters defining
/// how it should be parsed.
///
/// The first step in parsing command is called _resolving_ it. In this
/// step, the dispatcher does a traversal of the command graph while parsing
/// the input, until the input is empty and it has reached an executable node.
/// At this point, it invokes the `execute` function at the final node, passing
/// it the full command input.
///
/// Critically, the above step does't actually parse command arguments into
/// their final types: instead, it only knows about three types of arguments
/// (`SingleWord`, `Quoted`, and `Greedy`). This is the minimum needed for the dispatcher
/// to be able to determine which nodes to follow.
///
/// The _actual_ parsing is handled by the command's `execute()` function, which takes
/// the whole command input as a raw string. It's free to do whatever it likes to parse
/// the command, and then it uses that parsed data to do whatever it needs to do.
///
/// The benefit of the above process is this: the dispatcher doesn't have to worry about
/// parameter types and such; all it needs is some C-like raw data. Thereby we avoid
/// using generics and fancy dynamic dispatch, both of which would hinder the development
/// of a WASM command API.
///
/// Note that the `Node` API isn't intended to be used directly by users. Instead, the `command`
/// proc macro (unimplemented) and the `warp`-like builder API (unimplemented, TODO - Defman)
/// provide a convenient, elegant abstraction over raw command nodes.
pub struct CommandDispatcher<Ctx> {
    nodes: Vec<Node<Ctx>>,
    /// Maps literal names (root child nodes)
    /// to their node IDs. Used to accelerate the first step
    /// of parsing (which is the only one likely to be a bottleneck).
    root_literals: HashMap<String, NodeId>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InputConsumer {
    /// Consume until we reach a space or the end of input.
    SingleWord,
    /// If the input begins with a quote (single or double),
    /// then parse a string until the end quote. Otherwise,
    /// consume input as if this were `SingleWord`.
    Quoted,
    /// Consume all remaining input.
    Greedy,
}

/// The function used to execute a command.
pub type CommandFn<Ctx> = fn(ctx: &mut Ctx, input: &str) -> Result<()>;

pub struct Node<Ctx> {
    /// How to consume input.
    pub consumer: InputConsumer,

    /// Whether this node is a `literal` or `argument` node.
    pub kind: NodeKind,

    /// Whether this node is "executable."
    ///
    /// When the end of input is reached, the last node
    /// visited will have its execute function invoked.
    pub execute: Option<CommandFn<Ctx>>,

    /// Child nodes. After this node is consumed,
    /// parsing will move on to the children if there is remaining
    /// input.
    pub children: Vec<NodeId>,
}

pub enum NodeKind {
    Argument {
        /// Descriptor for the parser for this
        /// node. Used to build the Declare Commands packet.
        parser: ParserKind,
    },
    Literal(String),
}

/// Describes a parser from [this list](https://wiki.vg/Command_Data#Parsers).
/// Doesn't actually provide a means to parse functions—this is only
/// used to build the Declare Commands packet.
pub enum ParserKind {
    Bool,
    Double(RangeInclusive<f64>),
    Float(RangeInclusive<f32>),
    Integer(RangeInclusive<i32>),
    String,
    Entity {
        /// Whether only one entity is allowed.
        only_one: bool,
        /// Whether only players will be included.
        player_required: bool,
    },
    GameProfile,
    BlockPos,
    ColumnPos,
    Vec3,
    Vec2,
    BlockState,
    BlockPredicate,
    ItemStack,
    ItemPredicate,
    Color,
    ChatComponent,
    Message,
    JsonNbt,
    NbtPath,
    Objective,
    ObjectiveCritera,
    Operation,
    Particle,
    Rotation,
    ScoreboardSlot,
    ScoreHolder {
        /// Whether more than one entity will be allowed.
        multiple_allowed: bool,
    },
    Swizzle,
    Team,
    ItemSlot,
    ResourceLocation,
    MobEffect,
    Function,
    EntityAnchor,
    Range {
        decimals_allowed: bool,
    },
    IntRange,
    FloatRange,
    ItemEnchantment,
    EntitySummon,
    Dimension,
    Uuid,
    NbtTag,
    NbtCompoundTag,
    Time,
}

impl<Ctx> Default for CommandDispatcher<Ctx> {
    fn default() -> Self {
        Self::new()
    }
}

impl InputConsumer {
    /// Consumes the provided string according to the rules
    /// defined by this `InputConsumer`. After this call returns `Ok`,
    /// `input` will point to the remaining input and the returned
    /// string is the consumed input.
    pub fn consume<'a>(self, input: &mut &'a str) -> Result<&'a str> {
        let (start, chars_to_consume, chars_to_skip) = match self {
            InputConsumer::SingleWord => (0, find_space_position(input), 0),
            InputConsumer::Quoted => {
                if input.chars().next() == Some('"') {
                    let end_quote = input.chars().skip(1).position(|c| c == '"');

                    if let Some(end_quote) = end_quote {
                        // skip the quotes
                        (1, end_quote + 1, 1)
                    } else {
                        return Err(Error::Syntax(SyntaxError::UnterminatedString));
                    }
                } else {
                    (0, find_space_position(input), 0)
                }
            }
            InputConsumer::Greedy => (0, input.len(), 0),
        };

        let consumed = &input[start..chars_to_consume];
        *input = &input[chars_to_consume + chars_to_skip..];
        Ok(consumed)
    }
}

fn find_space_position(input: &str) -> usize {
    let space = input.chars().position(|c| c == ' ');

    if let Some(index) = space {
        index
    } else {
        // all remaining input
        input.len()
    }
}

impl<Ctx> CommandDispatcher<Ctx> {
    /// Creates a new `CommandDispatcher` with no nodes.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            root_literals: HashMap::new(),
        }
    }

    /// Adds a new node as a child of the provided node
    /// or as a child of the root if `parent` is set to `None`.
    pub fn add_node(&mut self, parent: Option<NodeId>, node: Node<Ctx>) -> NodeId {
        let id = NodeId(self.nodes.len());

        if let NodeKind::Literal(literal) = &node.kind {
            if parent.is_none() {
                self.root_literals.insert(literal.clone(), id);
            }
        }

        self.nodes.push(node);

        if let Some(parent_id) = parent {
            self.nodes[parent_id].children.push(id);
        }
        id
    }

    /// Parses and executes a command.
    pub fn execute(&self, full_command: &str, ctx: &mut Ctx) -> Result<()> {
        let mut command = full_command;
        let root_literal = InputConsumer::SingleWord.consume(&mut command)?;

        let root_node = match self.root_literals.get(root_literal) {
            Some(&node) => node,
            None => return Err(Error::Syntax(SyntaxError::UnknownCommand)),
        };
        let mut node_stack = self.nodes[root_node].children.to_vec();

        // Depth-first search to determine which node to execute.
        while let Some(node_id) = node_stack.pop() {
            let node = &self.nodes[node_id];
            let node_input = node.consumer.consume(&mut command)?;

            // Consume remaining whitespace
            while command.get(0..1) == Some(" ") {
                command = &command[1..];
            }

            match &node.kind {
                NodeKind::Literal(lit) => {
                    if node_input != lit {
                        continue;
                    }
                }
                NodeKind::Argument { .. } => (),
            }

            // If there's no remaining input, then we execute this node.
            if command.is_empty() {
                match &node.execute {
                    Some(execute) => return execute(ctx, full_command),
                    None => return Err(Error::Syntax(SyntaxError::MissingArgument)),
                }
            }

            // Push the node's children to the stack.
            for &child in &node.children {
                node_stack.push(child);
            }
        }

        Err(Error::Syntax(SyntaxError::MissingArgument))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consume_single_word() {
        let mut s = &mut "input after space";
        assert_eq!(InputConsumer::SingleWord.consume(&mut s).unwrap(), "input");
        assert_eq!(*s, " after space");
    }

    #[test]
    fn consume_single_word_without_trailing_space() {
        let mut s = &mut "input";
        assert_eq!(InputConsumer::SingleWord.consume(&mut s).unwrap(), "input");
        assert!(s.is_empty());
    }

    #[test]
    fn consume_quoted() {
        let mut s = &mut "\"in quotes\" not in quotes";
        assert_eq!(InputConsumer::Quoted.consume(&mut s).unwrap(), "in quotes");
        assert_eq!(*s, " not in quotes");
    }

    #[test]
    fn consume_quoted_without_quotes() {
        let mut s = &mut "not in quotes";
        assert_eq!(InputConsumer::Quoted.consume(&mut s).unwrap(), "not");
        assert_eq!(*s, " in quotes");
    }

    #[test]
    fn undelimited_quote() {
        let mut s = &mut "\"not delimited";
        let err = InputConsumer::Quoted.consume(&mut s).unwrap_err();
        assert!(matches!(
            err,
            Error::Syntax(SyntaxError::UnterminatedString)
        ));
    }
}
