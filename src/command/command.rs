use std::marker::PhantomData;

use anyhow::bail;

use crate::{generic::Func, parser::IterParser};

#[derive(Clone, Copy, Default, PartialEq, Eq, std::hash::Hash, Debug)]
pub struct CommandId {
    pub(crate) id: usize,
}

impl CommandId {
    pub fn of(value: usize) -> Self {
        Self { id: value }
    }
}

pub trait Command {
    type GameState;
    type CommandResult;
    fn call(&self, gamestate: Self::GameState, input: &str) -> anyhow::Result<Self::CommandResult>;
    fn regex(&self) -> String;
}

pub struct CommandSpec<GameState, CommandResult, F1, F2, P> {
    pub(crate) parser: P,
    pub(crate) mapping: F1,
    pub(crate) gamestate: PhantomData<GameState>,
    pub(crate) command_result: PhantomData<CommandResult>,
    pub(crate) mapping_result: PhantomData<F2>,
}

impl<CommandResult, P: IterParser, GameState, F1, F2, Ext> Command
    for CommandSpec<GameState, CommandResult, F1, F2, P>
where
    F1: Func<Ext, Output = F2>,
    F2: Func<GameState, Output = CommandResult>,
    P: IterParser<Extract = Ext>,
{
    type GameState = GameState;
    type CommandResult = CommandResult;

    fn regex(&self) -> String {
        self.parser.regex()
    }

    fn call(&self, gamestate: GameState, input: &str) -> anyhow::Result<CommandResult> {
        let mut state = P::ParserState::default();
        loop {
            match self.parser.parse(state, input) {
                (Ok((ext, _)), _) => return Ok(self.mapping.call(ext).call(gamestate)),
                (Err(_), None) => {
                    bail!("Not able to parse input");
                }
                (Err(_), Some(next_state)) => state = next_state,
            }
        }
    }
}
