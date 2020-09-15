pub mod dispatcher;
mod error;

#[doc(inline)]
pub use dispatcher::CommandDispatcher;
pub use error::{Error, Result, SyntaxError};
