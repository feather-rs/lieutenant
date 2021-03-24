use super::stateid::StateId;
use super::{dfa::DFA, NFA};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    iter,
};

// This file implements early termination for dfa's. If we at some state in the
// dfa only could get there by beeing on a spesiffic command, then we can mark the node as
// early terminating.

// How do we calculate this?

// 1) For every command C, we can create a nfa. Assosiate all the ends with a C_end id, and
//    the non ends with C_mid.
//
// 2) Combine all the command nfa's by or-ing them together. Then convert it to a dfa.
//
// 3) If any state in the dfa contains only ids from one command, then we cut off all of its outgoing edges
//    and set the assosiated value to an apropriate C_end value. Thus signaling that a failed dfa.find() -> Err(state_id)
//    might actually be a purpusfull termination.
//
// 4 a) Do a dfs to find all states reachable from the start. Remove the nodes that we cant reach.
// 4 b) Remove all unused byteclasses (Edges).

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum CmdPos<C: Copy + std::hash::Hash + PartialEq + Eq> {
    End(C),
    Mid(C),
}
impl<C: Copy + std::hash::Hash + PartialEq + Eq> CmdPos<C> {
    fn value(&self) -> &C {
        match self {
            CmdPos::End(c) => c,
            CmdPos::Mid(c) => c,
        }
    }

    fn is_end(&self) -> bool {
        match self {
            CmdPos::End(_) => true,
            CmdPos::Mid(_) => false,
        }
    }
}

impl<C: Copy + std::hash::Hash + Eq + std::fmt::Debug> NFA<CmdPos<C>> {
    pub fn from_command_regex(regex: &str, id: C) -> anyhow::Result<Self> {
        let mut nfa = NFA::<CmdPos<C>>::regex(regex)?;
        nfa.assosiate_ends(CmdPos::End(id));
        nfa.asssosiate_non_ends(CmdPos::Mid(id));
        Ok(nfa)
    }

    pub fn into_early_termination_dfa(self) -> DFA<CmdPos<C>> {
        if self.ends.len() < 2 {
            let dfa: DFA<CmdPos<C>> = self.into();
            return dfa;
        }

        let mut nfa_to_dfa: BTreeMap<BTreeSet<StateId>, StateId> = BTreeMap::new();
        let mut dfa = DFA::new();

        let start_ids = self.epsilon_closure(iter::once(StateId::of(0)).collect::<BTreeSet<_>>());

        // Create the start state of the DFA by taking the epsilon_closure of the start state of the NFA.
        let mut stack: Vec<BTreeSet<StateId>> = vec![start_ids.clone()];

        let dfa_id = dfa.push_state();
        nfa_to_dfa.insert(start_ids.clone(), dfa_id);

        let assosiated_values = {
            let mut values = HashSet::new();
            for id in start_ids.iter() {
                let state = &self[id.clone()];
                for a in state.assosiations.iter() {
                    values.insert(a.clone());
                }
            }
            values
        };
        dfa.assosiate(dfa_id, assosiated_values);

        while let Some(nfa_ids) = stack.pop() {
            let mut transitions = Vec::with_capacity(256);
            let is_end = nfa_ids.iter().any(|id| self.is_end(id));
            let dfa_id = nfa_to_dfa[&nfa_ids];

            let unique_assoc: Option<C> = {
                let mut unique_assoc: Option<C> = None;
                for state in nfa_ids.iter().map(|id| self[*id].clone()) {
                    let assocs = state.assosiations;

                    if assocs.len() > 1 {
                        unique_assoc = None;
                        break;
                    }

                    match (unique_assoc, assocs.iter().next()) {
                        (None, None) => {
                            unique_assoc = None;
                        }
                        (None, Some(x)) => unique_assoc = Some(*x.value()),
                        (Some(x), None) => unique_assoc = Some(x),
                        (Some(a), Some(b)) => {
                            if a != *b.value() {
                                unique_assoc = None;
                                break;
                            }
                        }
                    }
                }
                unique_assoc
            };

            if let Some(c) = unique_assoc {
                // Then we can make this dfa state a early termination.
                dfa.assosiate(dfa_id, iter::once(CmdPos::End(c)).collect());
                dfa.set_transitions(dfa_id.clone(), vec![None; 256]);
                continue;
            }

            // For each possible input symbol
            for b in 0..=255 as u8 {
                // Apply move to the newly-created state and the input symbol; this will return a set of states.
                let move_states = self.go(&nfa_ids, b);

                if move_states.is_empty() {
                    transitions.push(None);
                    continue;
                }

                // Apply the epsilon_closure to this set of states, possibly resulting in a new set.
                let move_state_e = self.epsilon_closure(move_states);

                let dfa_e_id = if let Some(dfa_e_id) = nfa_to_dfa.get(&move_state_e) {
                    dfa_e_id.clone()
                } else {
                    let dfa_e_id = dfa.push_state();
                    nfa_to_dfa.insert(move_state_e.clone(), dfa_e_id.clone());

                    let assosiated_values = {
                        let mut values = HashSet::new();
                        for id in move_state_e.iter() {
                            let state = &self[id.clone()];
                            for a in state.assosiations.iter() {
                                values.insert(a.clone());
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

            dfa.set_transitions(dfa_id.clone(), transitions);
            if is_end {
                dfa.push_end(dfa_id.clone());
            }
        }

        dfa.dedup_ends();
        dfa
    }
}

impl<C: Copy + std::hash::Hash + Eq + std::fmt::Debug> DFA<CmdPos<C>> {
    pub fn early_termination_find(&self, input: &str) -> Result<Vec<C>, Vec<C>> {
        match self.find(input) {
            Ok(id) => {
                let x = self
                    .assosiations(id)
                    .into_iter()
                    .filter(|cp| cp.is_end())
                    .map(|cp| *cp.value())
                    .collect();
                Ok(x)
            }
            Err(id) => match id {
                Some(id) => {
                    let ends: Vec<C> = self
                        .assosiations(id)
                        .into_iter()
                        .filter(|cp| cp.is_end())
                        .map(|cp| *cp.value())
                        .collect();
                    if ends.len() > 0 {
                        Ok(ends)
                    } else {
                        Err(self
                            .assosiations(id)
                            .into_iter()
                            .map(|cp| *cp.value())
                            .collect())
                    }
                }
                None => Err(vec![]),
            },
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::regex::{dfa::DFA, NFA};

    use super::CmdPos;

    #[test]
    fn simple_assosiated_value_check_literal() {
        let nfa = NFA::<CmdPos<usize>>::from_command_regex("his", 0).unwrap();
        let nfa = nfa
            .or(NFA::<CmdPos<usize>>::from_command_regex("hos", 1).unwrap())
            .unwrap();

        let dfa: DFA<CmdPos<usize>> = nfa.into_early_termination_dfa();
        println!("{:?}", dfa);

        match dfa.early_termination_find("hi") {
            Ok(_) => {}
            Err(_) => {
                assert!(false)
            }
        }
    }

    #[test]
    fn simple() {
        let regex = "hello world";
        let nfa = NFA::<CmdPos<usize>>::from_command_regex(regex, 0).unwrap();
        let dfa = nfa.into_early_termination_dfa();
        assert!(dfa.early_termination_find("hello world").is_ok());
        assert!(dfa.early_termination_find("no").is_err())
    }
}
