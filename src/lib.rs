mod command;
mod dispatcher;
mod parser;
mod provider;

pub use command::{Argument, Command, CommandSpec};
pub use dispatcher::CommandDispatcher;
pub use lieutenant_macros::{command, provider};
pub use parser::{ArgumentKind, Input};
pub use provider::{Provideable, Provider};

/// Result type returned by commands.
///
/// We use `std`'s `Result` instead of our
/// own `enum CommandResult` because of the
/// lack of a `Try` trait in stable.
///
/// The possible outcomes of a command are:
/// * `Ok`: the command succeeded and completed.
/// * `Pending`: the command cannot yet complete, so parsing
/// must be deferred until after it completes.
/// * `Error`: the command failed, i.e. user
/// input was wrong, or some internal error occurred.
pub type CommandResult<C, OK = <C as Context>::Ok> = Result<OK, CommandError<C>>;

/// An error returned by a command.
pub enum CommandError<C: Context> {
    /// An error occurred while executing the command.
    Error(C::Error),
    /// The command cannot complete immediately.
    /// This can be used to implement async commands.
    Pending(C::Pending),
}

/// Denotes a type that may be passed to commands as input.
pub trait Context: Send + Sync + 'static {
    /// The comprehensive error type returned
    /// when a command, parser, or provider fails.
    type Error: Send;

    /// The type returned when a command succeeds.
    type Ok;

    /// The type returned when a command returns `Pending`,
    /// meaning it cannot complete immediately. This
    /// is only useful when you want `async fn` commands
    /// or parsers. Otherwise, you may set this to the empty
    /// tuple: `()`.
    type Pending;

    /// The argument payload, i.e. the data each argument
    /// kind must expose. This must implement `FromBrigadierId`.
    ///
    /// If you don't need this type, set it to the empty tuple, `()`.
    type ArgumentPayload: FromBrigadierId;
}

/// The types of arguments supported by Brigadier,
/// which `lieutenant` aims to support and mimic.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BrigadierId {
    /// A boolean argument type (`"brigadier:bool"`)
    Bool,
    /// A double argument type (`"brigadier:double"`)
    Float64,
    /// A float argument type  (`"brigadier:float"`)
    Float32,
    /// A signed integer argument type (`"brigadier:integer"`)
    Integer,
    /// A string argument type (`"brigadier:string"`)
    String,
}

/// For argument payloads, converts from a `BrigadierId`
/// to an argument payload.
pub trait FromBrigadierId {
    fn from_brigadier_id(id: BrigadierId) -> Self;
}

impl FromBrigadierId for () {
    fn from_brigadier_id(_id: BrigadierId) -> Self {
        ()
    }
}
