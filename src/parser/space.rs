use super::IterParser;
use anyhow::anyhow;

#[derive(Debug)]
pub enum OnceState {
    More,
    Done,
}
impl Default for OnceState {
    fn default() -> Self {
        OnceState::More
    }
}

pub struct OneOrMoreSpace;
pub struct MaybeSpaces;

impl MaybeSpaces {
    pub fn new() -> Self {
        Self {}
    }
}

impl OneOrMoreSpace {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for OneOrMoreSpace {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MaybeSpaces {
    fn default() -> Self {
        Self::new()
    }
}

impl IterParser for OneOrMoreSpace {
    type Extract = ();

    type ParserState = ();

    #[allow(clippy::type_complexity)]
    fn parse<'p>(
        &self,
        _state: Self::ParserState,
        input: &'p str,
    ) -> (
        anyhow::Result<(Self::Extract, &'p str)>,
        Option<Self::ParserState>,
    ) {
        let before_len = input.len();
        let out = input.trim_start();
        if out.len() == before_len {
            (Err(anyhow!("Expected a space")), None)
        } else {
            (Ok(((), out)), None)
        }
    }

    fn regex(&self) -> String {
        "\\s+".to_string()
    }
}

impl IterParser for MaybeSpaces {
    type Extract = ();
    type ParserState = ();

    #[allow(clippy::type_complexity)]
    fn parse<'p>(
        &self,
        _state: Self::ParserState,
        input: &'p str,
    ) -> (
        anyhow::Result<(Self::Extract, &'p str)>,
        Option<Self::ParserState>,
    ) {
        let out = input.trim_start();
        (Ok(((), out)), None)
    }

    fn regex(&self) -> String {
        "\\s*".to_string()
    }
}
