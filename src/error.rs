pub type Result<T> = std::result::Result<T, Error>;

/// An error returned from command parsing or execution.
///
/// There are two types of errors:
/// * _Syntax errors_, when the parsers for a command throw an error,
/// or the input doesn't satisfy any command.
/// * _Semantic errors_, when a command throws an error. For example,
/// "player does not exist" is a semantic error.
///
/// For convencience, this type implements `From<T> where T: std::error::Error`
/// so that it can be used with the `?` operator. This will convert the error
/// into a _semantic_ error. If that's not what you want, you should do the
/// conversion explicitly.
#[derive(Debug)]
pub enum Error {
    ///
    /// A command cannot be parsed because its syntax is incorrect.
    Syntax(SyntaxError),
    /// A command could not be executed.
    Semantic(anyhow::Error),
}

impl<T> From<T> for Error
where
    anyhow::Error: From<T>,
{
    fn from(error: T) -> Self {
        Error::Semantic(error.into())
    }
}

/// Error returned when a command could not be parsed.
#[derive(Debug, thiserror::Error)]
pub enum SyntaxError {
    #[error("unterminated quoted string")]
    UnterminatedString,
    #[error("too little input")]
    MissingArgument,
    #[error("too many arguments")]
    TooManyArguments,
    #[error("unknown command")]
    UnknownCommand,

    #[error(transparent)]
    Custom(anyhow::Error),
}
