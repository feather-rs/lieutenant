use crate::generic::Func;
use crate::parser::parser::IterParser;

pub struct Map<P, F> {
    pub(crate) parser: P,
    pub(crate) map: F,
}
#[derive(Debug)]
pub struct MapState<S> {
    state: S,
}

impl<S: Default> Default for MapState<S> {
    fn default() -> Self {
        MapState {
            state: S::default(),
        }
    }
}

impl<P, F> IterParser for Map<P, F>
where
    P: IterParser,
    F: Func<P::Extract>,
{
    type Extract = (F::Output,);
    type ParserState = P::ParserState;

    #[allow(clippy::type_complexity)]
    fn parse<'p>(
        &self,
        state: Self::ParserState,
        input: &'p str,
    ) -> (
        anyhow::Result<(Self::Extract, &'p str), anyhow::Error>,
        Option<Self::ParserState>,
    ) {
        let (result, state) = self.parser.parse(state, input);

        match result {
            Ok((ext, out)) => (Ok(((self.map.call(ext),), out)), state),
            Err(err) => (Err(err), state),
        }
    }
    fn regex(&self) -> String {
        self.parser.regex()
    }
}
