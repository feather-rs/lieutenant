// http://www.cs.nuim.ie/~jpower/Courses/Previous/parsing/node9.html

use super::*;
use dfa::DFA;
use nfa::NFA;
use stateid::StateId;
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fmt::Debug,
    iter,
};

impl<A: Copy + Eq + std::hash::Hash + Debug> NFA<A> {
    /// Goes through all the states in 'from' and adds all the states one can get to by using epsilon
    /// transitions.
    pub(crate) fn epsilon_closure(&self, from: BTreeSet<StateId>) -> BTreeSet<StateId> {
        let mut stack: Vec<_> = from.into_iter().collect();
        let mut states = BTreeSet::new();
        while let Some(q) = stack.pop() {
            states.insert(q);
            for q_e in self[q].epsilons.iter() {
                if states.insert(*q_e) {
                    stack.push(*q_e)
                }
            }
        }
        states
    }

    pub(crate) fn go(&self, states: &BTreeSet<StateId>, byte: u8) -> BTreeSet<StateId> {
        states
            .iter()
            .cloned()
            .flat_map(|id| self[(id, byte)].iter().cloned())
            .collect()
    }
}

impl<A: Eq + std::hash::Hash + Copy + Debug> From<NFA<A>> for DFA<A> {
    fn from(nfa: NFA<A>) -> Self {
        let mut nfa_to_dfa: BTreeMap<BTreeSet<StateId>, StateId> = BTreeMap::new();
        let mut dfa = DFA::new();

        let start_ids = nfa.epsilon_closure(iter::once(StateId::of(0)).collect::<BTreeSet<_>>());

        // Create the start state of the DFA by taking the epsilon_closure of the start state of the NFA.
        let mut stack: Vec<BTreeSet<StateId>> = vec![start_ids.clone()];

        let dfa_id = dfa.push_state();
        nfa_to_dfa.insert(start_ids.clone(), dfa_id);

        let assosiated_values = {
            let mut values = HashSet::new();
            for id in start_ids.iter() {
                let state = &nfa[*id];
                for a in state.assosiations.iter() {
                    values.insert(*a);
                }
            }
            values
        };
        dfa.assosiate(dfa_id, assosiated_values);

        while let Some(nfa_ids) = stack.pop() {
            let mut transitions = Vec::with_capacity(256);
            let is_end = nfa_ids.iter().any(|id| nfa.is_end(id));
            let dfa_id = nfa_to_dfa[&nfa_ids];

            // For each possible input symbol
            for b in 0..=255_u8 {
                // Apply move to the newly-created state and the input symbol; this will return a set of states.
                let move_states = nfa.go(&nfa_ids, b);

                if move_states.is_empty() {
                    transitions.push(None);
                    continue;
                }

                // Apply the epsilon_closure to this set of states, possibly resulting in a new set.
                let move_state_e = nfa.epsilon_closure(move_states);

                let dfa_e_id = if let Some(dfa_e_id) = nfa_to_dfa.get(&move_state_e) {
                    *dfa_e_id
                } else {
                    let dfa_e_id = dfa.push_state();
                    nfa_to_dfa.insert(move_state_e.clone(), dfa_e_id);

                    let assosiated_values = {
                        let mut values = HashSet::new();
                        for id in move_state_e.iter() {
                            let state = &nfa[*id];
                            for a in state.assosiations.iter() {
                                values.insert(*a);
                            }
                        }
                        values
                    };
                    dfa.assosiate(dfa_e_id, assosiated_values);

                    // Each time we generate a new DFA state, we must apply step 2 to it. The process is complete when applying step 2 does not yield any new states.
                    stack.push(move_state_e);
                    dfa_e_id
                };

                transitions.push(Some(dfa_e_id));
            }

            dfa.set_transitions(dfa_id, transitions);
            if is_end {
                dfa.push_end(dfa_id);
            }
        }

        dfa.dedup_ends();
        dfa
    }
}

#[cfg(test)]
mod tests {

    use crate::regex::qc::NFAQtCase;
    use crate::regex::{dfa::DFA, NFA};

    #[test]
    fn simple_assosiated_value_check_literal() {
        let mut nfa = NFA::<usize>::literal("hi");
        nfa.assosiate_ends(42);

        let dfa: DFA<usize> = nfa.into();

        match dfa.find("hi") {
            Ok(x) => {
                assert!(dfa[x].is_assosiated_with(&42))
            }
            Err(_) => {
                panic!()
            }
        }
    }

    #[quickcheck]
    fn qc_nfa_to_dfa(mut case: NFAQtCase) -> bool {
        case.nfa.assosiate_ends(42);

        let dfa: DFA<usize> = case.nfa.clone().into();

        for x in case.matches.iter() {
            match dfa.find(x.as_str()) {
                Ok(x) => {
                    if !dfa[x].is_assosiated_with(&42) {
                        println!("HEYOOOO {:?}", case.nfa);
                        println!("Wooooo: {:?}", x);
                        println!("Woot?: {:?}", dfa[x]);
                        println!("Buga:nfa {:?}", dfa);
                        return false;
                    }
                }
                Err(_) => {
                    return false;
                }
            }
        }

        true
    }
}
