// use std::collections::HashMap;

// use indexmap::IndexSet;

// use crate::{byteclass::{ByteClass, ByteClassId}, dfa::{DFA, DfaState}};

// impl <A: Eq + std::hash::Hash> DfaState<A> {

//     // When canonicalizing a dfa, we want to reduce the number of byteclasses.
//     // The order of self.table is arbitrary, so we can change it to suite ourselves.
//     // For every byteclass we can define a reordering such that [0,0,3,2,2,1,4 ...]
//     // is turned into [0,0,1,2,2,3,4 ...]. By mapping 3 to 1, 2 to 2, 1 to 3, and 4 to 4.
//     // This function applies this mapping to self.table
//     pub(crate) fn canonicalize_table(&mut self, mappings: &HashMap<ByteClassId, Vec<u8>>) {
//         let mapping = &mappings[&self.class];
//         for (index, value) in mapping.iter().enumerate() {
//             self.table.swap(index, *value as usize)
//         }
//     }

// }

// impl<A: std::hash::Hash + Eq> DFA<A> {

//     fn minimiza_byteclass_count(&mut self) {
//         let mut order_mappings : HashMap<ByteClassId, Vec<u8>> = HashMap::with_capacity(self.transitions.len());
//         for (id, byteclass) in self.transitions.iter().enumerate() {
//             order_mappings.insert(ByteClassId::of(id), byteclass.chronologization_map());
//         }

//         for state in self.states.iter_mut() {
//             state.canonicalize_table(&order_mappings);
//         }

//         let id_mapping = HashMap::<ByteClassId, ByteClassId>::with_capacity(self.transitions.len());
//         let new_transitions = IndexSet::<ByteClass>::new();

//         for (id, byteclass) in self.transitions.iter().enumerate() {

//         }

//         for state in self.states.iter_mut() {
//             let old_byteclass_id = state.class;
//             let mapping =
//         }

//     }

// }
