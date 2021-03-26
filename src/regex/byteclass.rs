use std::{
    fmt,
    ops::{Index, IndexMut},
};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub(crate) struct ByteClassId(pub u16);

impl From<u16> for ByteClassId {
    fn from(v: u16) -> Self {
        Self(v)
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub(crate) struct ByteClass([u8; 256]);

impl ByteClass {
    pub(crate) fn empty() -> Self {
        ByteClass([0; 256].into())
    }

    pub(crate) fn set(&mut self, index: u8, value: u8) {
        self.0[index as usize] = value;
    }
}

impl From<u8> for ByteClass {
    fn from(value: u8) -> Self {
        let mut values = [0u8; 256];
        values[value as usize] = 1;
        Self(values.into())
    }
}

impl fmt::Debug for ByteClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:?}", self.0)?;
        Ok(())
    }
}

impl Index<u8> for ByteClass {
    type Output = u8;
    fn index(&self, index: u8) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<u8> for ByteClass {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}
