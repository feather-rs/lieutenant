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
pub(crate) struct ByteClass(Vec<u8>);

impl ByteClass {
    pub(crate) fn empty() -> Self {
        ByteClass([0; 256].into())
    }

    pub(crate) fn set(&mut self, index: u8, value: u8) {
        self.0[index as usize] = value;
    }

    // When we convert from dfa to nfa, we want to reduce the number of states.
    // we therefor do a chronologization step. We want similar byteclasses be merged.
    // vec![0, 0, 4, 1, 1, 2, 3 , 0 ...] -> vec![0, 0, 1, 2, 2, 3, 4, 0 ...]
    // this method calculates the mapping for our  chronologization.
    // pub(crate) fn chronologization_map(&self) -> Vec<u8> {
    //     let mut map = HashMap::<u8, u8>::new();
    //     let mut c = 0;
    //     for v in self.0.iter() {
    //         if !map.contains_key(v) {
    //             map.insert(*v, c);
    //             c += 1;
    //         }
    //     }

    //     let mut result = Vec::with_capacity(map.len());
    //     for i in 0..map.len() {
    //         result.push(map[&(i as u8)])
    //     }

    //     result
    // }

    // pub(crate) fn chronologize_by(&self, mapping: Vec<u8>) -> Self {
    //     let mut result = self.clone();

    //     for v in result.0.iter_mut() {
    //         *v = mapping[*v as usize]
    //     }

    //     result
    // }
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
        writeln!(f, "{:?}", self.0.to_vec())?;
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
