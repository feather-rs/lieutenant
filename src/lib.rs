mod command;
mod dispatcher;
mod parser;

pub use command::{Command, CommandSpec, Argument};
pub use dispatcher::CommandDispatcher;
pub use lieutenant_macros::command;
pub use parser::{parsers, ArgumentChecker, ArgumentKind, ArgumentParser, Input};

/// Denotes a type that may be passed to commands as input.
pub trait Context {
    type Error: std::error::Error + Send + Sync;
    type Ok;
}