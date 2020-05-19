pub(crate) mod generic;
mod command;
mod parser;
pub use parser::Input;

use std::error::Error;

pub trait Context: Clone + Copy {
    type Error: Error;
}