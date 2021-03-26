use indexmap::IndexSet;
//use regex_to_nfa::regex_to_nfa;
use super::byteclass::{ByteClass, ByteClassId};
use super::stateid::StateId;
use std::{
    collections::{BTreeSet, HashSet},
    convert::TryInto,
    fmt::Debug,
    hash, iter,
    mem::{self},
    ops::{Index, IndexMut, Range},
    usize,
};

#[derive(Debug, Clone)]
pub struct NfaState<A> {
    // The first element should always be present. This is so that the zeros in the byteclass point to something.
    // The first element would almost always be a empty vec.  This is so that the zeros in the byteclass would signal a failed pars.
    table: Vec<Vec<StateId>>,

    // A byteclass is basically a [u8; 256]. If it[6] = 1, then from the state 'self' there are outgoing edges containing 6. These edges go from
    // self to 'x' for x in self.table[1].
    class: ByteClassId,

    pub(crate) epsilons: Vec<StateId>,
    pub(crate) assosiations: HashSet<A>,
}

/// Returns the states one would get to from self if the input u8 was 'index'.
impl<A> Index<u8> for NfaState<A> {
    type Output = Vec<StateId>;
    fn index(&self, index: u8) -> &Self::Output {
        &self.table[index as usize]
    }
}

impl<A: Eq + std::hash::Hash> IndexMut<u8> for NfaState<A> {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.table[index as usize]
    }
}

impl<A: Eq + Debug + std::hash::Hash + Default + Copy> Default for NFA<A> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<A: Eq + std::hash::Hash> NfaState<A> {
    fn empty() -> Self {
        Self {
            table: vec![vec![]],
            class: ByteClassId(0),
            epsilons: vec![],
            assosiations: HashSet::new(),
        }
    }

    fn associate_with(&mut self, value: A) {
        self.assosiations.insert(value);
    }

    fn extend_association_with(&mut self, values: HashSet<A>) {
        self.assosiations.extend(values)
    }
}

#[derive(Debug, Clone)]
pub struct NFA<A: std::hash::Hash> {
    /// Represents the nodes in the NFA.
    pub(crate) states: Vec<NfaState<A>>,

    /// Sort of represents the edges in the NFA. Every NfaState has a Vec of its neighbours. The byteclass values are used as a ofsett
    /// into this neighbouring Vec. This is a space saving optimisation. The first value is always a completly empty byteclass.
    translations: IndexSet<ByteClass>,

    /// These are the termination states of the NFA. If a stream of bytes ends on one of these states, we consider it a
    /// sucsess.
    pub(crate) ends: Vec<StateId>,
}

impl<A: std::hash::Hash> Index<ByteClassId> for NFA<A> {
    type Output = ByteClass;
    fn index(&self, ByteClassId(index): ByteClassId) -> &Self::Output {
        &self.translations[index as usize]
    }
}

impl<A: std::hash::Hash> Index<StateId> for NFA<A> {
    type Output = NfaState<A>;
    fn index(&self, StateId(index): StateId) -> &Self::Output {
        &self.states[index as usize]
    }
}

impl<A: std::hash::Hash> IndexMut<StateId> for NFA<A> {
    fn index_mut(&mut self, StateId(index): StateId) -> &mut Self::Output {
        &mut self.states[index as usize]
    }
}

impl<A: std::hash::Hash> Index<(StateId, Option<u8>)> for NFA<A> {
    type Output = Vec<StateId>;
    fn index(&self, (id, step): (StateId, Option<u8>)) -> &Self::Output {
        let state = &self[id];
        match step {
            Some(b) => &state[self[state.class.clone()][b]],
            None => &state.epsilons,
        }
    }
}

impl<A: std::hash::Hash> Index<(StateId, u8)> for NFA<A> {
    type Output = Vec<StateId>;
    fn index(&self, (id, b): (StateId, u8)) -> &Self::Output {
        let state = &self[id];
        &state[self[state.class.clone()][b]]
    }
}

impl<A: Copy + Eq + std::hash::Hash + Debug> NFA<A> {
    /// Does not match anything, not even a enpty string.
    pub fn empty() -> Self {
        Self {
            states: vec![NfaState::empty()],
            translations: iter::once(ByteClass::empty()).collect::<IndexSet<ByteClass>>(),
            ends: vec![],
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        (self.states.len() == 1) && (self.translations.len() == 1) && (self.ends.len() == 0)
    }

    pub fn is_end(&self, id: &StateId) -> bool {
        self.ends.contains(id)
    }

    pub(crate) fn push_state(&mut self) -> StateId {
        self.states.push(NfaState::empty());
        StateId(self.states.len() as u32 - 1)
    }

    pub(crate) fn push_end(&mut self, state: StateId) {
        self.ends.push(state)
    }

    pub(crate) fn push_connection(&mut self, from: StateId, to: StateId, c: u8) {
        let mut state = self.index(from).clone();
        //state.push_connection(self,to,c);

        let byteclass = &self[state.class.clone()];

        match byteclass[c] {
            0 => {
                // This means that self has no existing connection for input 'c'.
                state.table.push(Vec::with_capacity(1));
                let index = state.table.len() - 1;
                state.table[index].push(to);

                // Update class
                let mut new_byteclass = byteclass.clone();
                new_byteclass.set(c, index as u8);
                let (class_index, _) = self.translations.insert_full(new_byteclass);
                state.class = ByteClassId::from(class_index as u16);

                // We could now run a GC like procedure on the nfa, because nfa.transitions might contain a unused byteclass.
                // this might be reasonable to do, but since we are anyways ging to discard the nfa and replace it by a dfa,
                // i would not worry about it to begin with.
            }
            x => {
                // This means that there already exists index
                state.table[x as usize].push(to);
            }
        };

        self[from] = state;
    }

    pub(crate) fn push_connections<Itr: IntoIterator<Item = u8>>(
        &mut self,
        from: StateId,
        to: StateId,
        values: Itr,
    ) {
        let existing_byteclass = &self[self[from].class.clone()];
        let mut new_byteclass = existing_byteclass.clone();

        for c in values {
            match new_byteclass[c] {
                0 => {
                    // This means that self has no existing connection for input c
                    self[from].table.push(Vec::with_capacity(1));
                    let index = self.states[from.0 as usize].table.len() - 1;
                    self[from].table[index].push(to);
                    // Cast is safe because len of table is always 256
                    new_byteclass[c] = index as u8;
                }
                x => {
                    // This means that there already exists a connection from self to 'to'
                    self[from].table[x as usize].push(to);
                }
            }
        }

        let (class_index, _) = self.translations.insert_full(new_byteclass);
        self[from].class = ByteClassId::from(class_index as u16);
    }

    pub(crate) fn push_epsilon(&mut self, from: StateId, to: StateId) {
        let state = self.index_mut(from);
        state.epsilons.push(to)
    }

    /// Makes every end state in self have a epsilon transition to the 'other' nfa.
    /// Note how the end states are modified, so this method does not preserve associated values.
    pub fn followed_by(&mut self, other: NFA<A>) -> anyhow::Result<()> {
        if self.is_empty() {
            *self = other;
            return Ok(());
        } else if other.is_empty() {
            return Ok(());
        }

        // If self has a single end state, then we can forgo adding a epsilon transition.
        // this optimisation is probably not nessesary to think about because in the convertion from nfa
        // to dfa it is optimised away anyways. It would however probably make that step go faster.

        let state_ofset = self.states.len();

        for other_index in 0..other.states.len() {
            let mut state = other.states[other_index].clone();

            // When adding the states from 'other' all the StateId's are shifted by 'state_ofset'
            state.table = state
                .table
                .iter_mut()
                .map(|vec| {
                    vec.iter_mut()
                        .map(|id| id.add(state_ofset))
                        .collect::<Vec<_>>()
                })
                .collect();

            // When adding the state from 'other' we need to add its byteclass. We can however not just shift the byteclassid by 'state_ofset', because
            // the byteclass might already be present in the 'self' nfa's  byteclass.
            let byteclass = other.index(state.class);
            let (i, _) = self.translations.insert_full(byteclass.clone());
            state.class = ByteClassId(i.try_into()?);

            // Update epsilons, by shifting them by state_ofset
            // @TODO handle convetion error from add.
            state.epsilons = state
                .epsilons
                .into_iter()
                .map(|e| e.add(state_ofset))
                .collect();

            self.states.push(state);
        }

        for old_end in self.ends.clone() {
            self.push_epsilon(old_end, StateId(state_ofset as u32));
        }
        self.ends = other
            .ends
            .into_iter()
            .map(|id| id.add(state_ofset))
            .collect();

        Ok(())
    }

    pub fn literal(lit: &str) -> Self {
        let mut nfa = NFA::empty();
        let mut prev = StateId(0);

        for c in lit.bytes() {
            let next = nfa.push_state();
            nfa.push_connection(prev, next, c);
            prev = next;
        }

        nfa.ends = vec![prev];
        nfa
    }

    // Returns ok, when there are further states to consume
    fn _find_step(
        &self,
        bytes: &[u8],
        mut current_states: BTreeSet<StateId>,
    ) -> (Result<BTreeSet<StateId>, BTreeSet<StateId>>, bool) {
        // Follow epsilons all the way.

        current_states = self.epsilon_closure(current_states);

        if bytes.is_empty() {
            if self.ends.iter().any(|x| current_states.contains(x)) {
                return (Ok(current_states), false);
            } else {
                return (Err(current_states), false);
            }
        }

        let new_states = self.go(&current_states, bytes[0]);

        if new_states.is_empty() && !bytes[1..].len() == 0 {
            return (Err(new_states), false);
        }

        (Ok(new_states), true)
    }

    pub fn _find<T: AsRef<[u8]>>(&self, text: T) -> Result<BTreeSet<StateId>, BTreeSet<StateId>> {
        let mut bytes = text.as_ref();
        let mut current_states = iter::once(StateId::of(0)).collect::<BTreeSet<_>>();
        loop {
            match self._find_step(bytes, current_states) {
                (Ok(x), false) => {
                    return Ok(x
                        .into_iter()
                        .filter(|x| self.ends.contains(x))
                        .collect::<BTreeSet<_>>())
                }
                (Err(x), false) => return Err(x),
                (Ok(x), true) => {
                    bytes = &bytes[1..];
                    current_states = x;
                }
                (Err(_), true) => unreachable!("unreachable state in nfa find"),
            }
        }
    }

    /// Zero or more matches.
    pub fn repeat(self) -> anyhow::Result<Self> {
        //    /–>––––––––––––––––––––––––––>\
        // >(a) -> (b) -> [self] -> (c) ->((d))
        //          \<-------------</

        let mut nfa = NFA::<A>::empty();
        let a = StateId(0);
        let b = nfa.push_state();
        let c = nfa.push_state();
        let d = nfa.push_state();

        nfa.push_epsilon(a, d);
        nfa.push_epsilon(a, b);
        nfa.ends = vec![b]; // Makes self connect with epsilon from b
        nfa.followed_by(self)?;

        // Makes self connecting to c
        let ends = mem::take(&mut nfa.ends);
        for end in ends {
            nfa.push_epsilon(end, c);
            let assocs = mem::take(&mut nfa[end].assosiations);
            nfa[d].extend_association_with(assocs);
        }

        nfa.push_epsilon(c, b);
        nfa.push_epsilon(c, d);

        nfa.ends.push(d);

        Ok(nfa)
    }

    pub fn or(self, other: NFA<A>) -> anyhow::Result<Self> {
        //    /–>––[self]
        // >(a)
        //   \–>––[other]

        let mut nfa = NFA::<A>::empty();
        let a = StateId(0);

        nfa.followed_by(self)?;

        let ends = nfa.ends.clone();
        nfa.ends = vec![a]; 
        nfa.followed_by(other)?;
        nfa.ends.extend(ends);

        Ok(nfa)
    }

    pub fn assosiate_ends(&mut self, value: A) {
        for e in &self.ends {
            let end = &mut self.states[e.0 as usize];
            end.associate_with(value);
        }
    }

    pub fn asssosiate_non_ends(&mut self, value: A) {
        let ends = &self.ends;
        for (id, state) in self.states.iter_mut().enumerate() {
            let state_id = StateId::of(id);
            if ends.contains(&state_id) {
                continue;
            }
            state.associate_with(value);
        }
    }

    fn _end_assosiations(&self) -> HashSet<A> {
        let mut result = HashSet::new();
        for e in &self.ends {
            let end = &self.states[e.0 as usize];
            result.extend(end.assosiations.clone());
        }

        result
    }
}

impl<A: Eq + hash::Hash + Copy + Debug> From<Range<u8>> for NFA<A> {
    fn from(range: Range<u8>) -> Self {
        let mut nfa = NFA::<A>::empty();
        let a = nfa.push_state();
        nfa.push_connections(StateId::of(0), a, range);
        nfa
    }
}

#[cfg(test)]
mod tests {

    use std::ops::Add;

    use super::*;
    #[test]
    fn literal() {
        let nfa = NFA::<usize>::literal("hello");
        assert!(nfa._find("hello").is_ok());
        assert!(nfa._find("ello").is_err());
        assert!(nfa._find("hhello").is_err());
        assert!(nfa._find("helloo").is_err());
        assert!(nfa._find("helo").is_err());
        assert!(nfa._find("hxllo").is_err());
    }

    #[test]
    fn empty() {
        // The empty nfa does not match the empty string.
        assert!(NFA::<usize>::empty()._find("").is_err());
    }

    #[test]
    fn empty_lit() {
        let nfa = NFA::<usize>::literal("");
        assert!(nfa._find("").is_ok());
    }

    #[test]
    fn empty_lit_ored_with_non_empty() {
        let nfa = NFA::<usize>::literal("")
            .or(NFA::<usize>::literal("a"))
            .unwrap();
        assert!(nfa._find("").is_ok());
        assert!(nfa._find("a").is_ok());
        assert!(nfa._find("b").is_err());
        assert!(nfa._find("aa").is_err());
    }

    #[quickcheck]
    fn literal_param(input: String) -> bool {
        let nfa = NFA::<usize>::literal(input.as_str());
        nfa._find(input.as_str()).is_ok()
    }

    #[test]
    fn empty_literal() {
        let nfa = NFA::<usize>::literal("");
        assert!(nfa._find("").is_ok());
    }

    #[quickcheck]
    fn literal_param_not_eq(input: String, other: String) -> bool {
        let nfa = NFA::<usize>::literal(input.as_str());
        nfa._find(other.as_str()).is_ok() == (input.eq(&other))
    }

    #[test]
    fn followed_by_eq() {
        let testcase = ["a", "abba", "abb", "ab", "aba", "b"];
        let mut testcase2 = testcase;
        testcase2.reverse();

        for head in testcase.iter() {
            for tail in testcase2.iter() {
                let mut head_nfa = NFA::<usize>::literal(head);
                let tail_nfa = NFA::<usize>::literal(tail);
                head_nfa.followed_by(tail_nfa).unwrap();
                assert!(
                    head_nfa._find(format!("{}{}", head, tail).as_str()).is_ok(),
                    format!("{}{}", head, tail)
                );
            }
        }
    }

    #[quickcheck]
    fn followed_by_param_eq(head: String, tail: String) -> bool {
        let mut head_nfa = NFA::<usize>::literal(head.as_str());
        let tail_nfa = NFA::<usize>::literal(tail.as_str());

        head_nfa.followed_by(tail_nfa).unwrap();
        head_nfa._find(head.add(tail.as_str()).as_str()).is_ok()
    }

    #[quickcheck]
    fn repeat_param0(lit: String) -> bool {
        let head_nfa = NFA::<usize>::literal(lit.as_str());
        let nfa = head_nfa.repeat().unwrap();
        nfa._find(format!("").as_str()).is_ok()
    }

    #[quickcheck]
    fn repeat_param1(lit: String) -> bool {
        let head_nfa = NFA::<usize>::literal(lit.as_str());
        let nfa = head_nfa.repeat().unwrap();
        nfa._find(lit).is_ok()
    }

    #[quickcheck]
    fn repeat_param2(lit: String) -> bool {
        let head_nfa = NFA::<usize>::literal(lit.as_str());
        let nfa = head_nfa.repeat().unwrap();
        nfa._find(format!("{}{}", lit.as_str(), lit.as_str()).as_str())
            .is_ok()
    }

    #[test]
    fn repeat() {
        let testcase = ["a", "b", "ab", "", " "];

        for t in testcase.iter() {
            let head_nfa = NFA::<usize>::literal(t);
            let nfa = head_nfa.repeat().unwrap();

            assert!(
                nfa._find(format!("").as_str()).is_ok(),
                format!("{} zero", t)
            );
            assert!(
                nfa._find(t.to_string().as_str()).is_ok(),
                format!("{} one", t)
            );
            assert!(
                nfa._find(format!("{}{}", t, t).as_str()).is_ok(),
                format!("{} two", t)
            );
        }
    }

    #[quickcheck]
    fn or_param1(a: String, b: String) -> bool {
        let nfa = NFA::<usize>::literal(a.as_str());
        let nfb = NFA::<usize>::literal(b.as_str());
        let or = nfa.or(nfb).unwrap();
        or._find(&a).is_ok() && or._find(&b).is_ok()
    }

    #[quickcheck]
    fn or_param2(a: String, b: String, c: String) -> bool {
        let nfa = NFA::<usize>::literal(a.as_str());
        let nfb = NFA::<usize>::literal(b.as_str());
        let or = nfa.or(nfb).unwrap();
        if a != c && b != c {
            or._find(&c).is_err()
        } else {
            true
        }
    }

    #[quickcheck]
    fn literal_param_ass(a: String, ass: usize) -> bool {
        let mut nfa = NFA::<usize>::literal(&a);
        nfa.assosiate_ends(ass);
        nfa._end_assosiations().contains(&ass)
    }

    #[quickcheck]
    fn literal_or_ass1(a: String, b: String, ass: usize, bss: usize) -> bool {
        if a.is_empty() || b.is_empty() {
            return true;
        }

        let mut nfa = NFA::<usize>::literal(&a);
        let mut nfb = NFA::<usize>::literal(&b);
        nfa.assosiate_ends(ass);
        nfb.assosiate_ends(bss);

        let or = nfa.or(nfb).unwrap();
        let asses = or._end_assosiations();
        asses.contains(&ass) && asses.contains(&bss)
    }

    #[quickcheck]
    fn literal_or_ass2(a: String, b: String, ass: usize, bss: usize) -> bool {
        if a.is_empty() || b.is_empty() {
            return true;
        }

        let mut nfa = NFA::<usize>::literal(&a);
        let mut nfb = NFA::<usize>::literal(&b);
        nfa.assosiate_ends(ass);
        nfb.assosiate_ends(bss);

        let or = nfa.or(nfb).unwrap();

        let a_ends = or._find(a.as_str()).unwrap();
        let b_ends = or._find(b.as_str()).unwrap();

        let a_end = a_ends.iter().find(|x| or.ends.contains(x)).unwrap();
        let b_end = b_ends.iter().find(|x| or.ends.contains(x)).unwrap();

        let x = &or[*a_end].assosiations.contains(&ass);
        let y = &or[*b_end].assosiations.contains(&bss);

        *x && *y
    }

    // #[quickcheck]
    // fn random_nfa_matches(case: NFAQtCase) -> bool{

    //     // Check that the case nfa matches all the strings it contains.
    //     for s in case.matches {
    //         if !case.nfa.find(s.as_str()).is_some(){
    //             return false;
    //         }
    //     }

    //     true

    // }
}
