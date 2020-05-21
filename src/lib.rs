mod command;
pub(crate) mod generic;
mod parser;
pub use parser::Input;

use std::error::Error;

pub trait Context: Clone {
    type Error: Error + From<command::CommandError>;
    type Ok: generic::Tuple;
}
