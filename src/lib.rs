mod command;
mod dispatcher;
mod parser;
mod provider;

pub use command::{Argument, Command, CommandSpec};
pub use dispatcher::CommandDispatcher;
pub use lieutenant_macros::{command, provider};
pub use parser::{parsers, ArgumentChecker, ArgumentKind, ArgumentParser, ParserUtil};
pub use provider::{Provideable, Provider};

/// Denotes a type that may be passed to commands as input.
pub trait Context: Send + 'static {
    type Error: std::error::Error + Send;
    type Ok;
}
