#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateId(pub u32);

impl StateId {
    pub fn of(id: usize) -> Self {
        assert!(id < u32::MAX as usize);
        Self(id as u32)
    }

    pub(crate) fn add(&self, n: usize) -> Self {
        Self::of(self.0 as usize + n)
    }
}
