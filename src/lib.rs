mod command;
mod dispatcher;
mod parser;
mod provider;

pub use command::{Argument, Command, CommandSpec};
pub use dispatcher::CommandDispatcher;
pub use lieutenant_macros::{command, provider};
pub use parser::{ArgumentKind, Input};
pub use provider::{Provideable, Provider};

/// Denotes a type that may be passed to commands as input.
pub trait Context: Send + Sync + 'static {
    type Error: Send;
    type Ok;
}
