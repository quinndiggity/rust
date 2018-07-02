// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use borrow_check::nll::constraints::{ConstraintIndex, ConstraintSet};
use rustc::ty::RegionVid;
use rustc_data_structures::graph;
use rustc_data_structures::indexed_vec::IndexVec;

/// An initial graph which contains a node for every constraint index.
/// This is basically just represented as a set of linked lists; for
/// each region R, there is a linked list of constraints that lead to
/// other regions. (The graph is always defined relative to some set.)
crate struct FullConstraintGraph<'s> {
    set: &'s ConstraintSet,
    first_constraints: IndexVec<RegionVid, Option<ConstraintIndex>>,
    next_constraints: IndexVec<ConstraintIndex, Option<ConstraintIndex>>,
}

impl<'s> FullConstraintGraph<'s> {
    /// Constraint a graph where each region constraint `R1: R2` is
    /// treated as an edge `R2 -> R1`. This is useful for cheaply
    /// finding dirty constraints.
    crate fn new(set: &'s ConstraintSet, num_region_vars: usize) -> Self {
        let mut first_constraints = IndexVec::from_elem_n(None, num_region_vars);
        let mut next_constraints = IndexVec::from_elem(None, &set.constraints);

        for (idx, constraint) in set.constraints.iter_enumerated().rev() {
            let mut head = &mut first_constraints[constraint.sup];
            let mut next = &mut next_constraints[idx];
            debug_assert!(next.is_none());
            *next = *head;
            *head = Some(idx);
        }

        Self { set, first_constraints, next_constraints }
    }

    crate fn sub_regions(
        &self,
        region_sup: RegionVid,
    ) -> Successors<'_> {
        let first = self.first_constraints[region_sup];
        Successors { graph: self, pointer: first }
    }
}

crate struct Successors<'s> {
    graph: &'s FullConstraintGraph<'s>,
    pointer: Option<ConstraintIndex>,
}

impl<'s> Iterator for Successors<'s> {
    type Item = RegionVid;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(p) = self.pointer {
            let r = self.graph.set[p].sub;
            self.pointer = self.graph.next_constraints[p];
            Some(r)
        } else {
            None
        }
    }
}

impl<'s> graph::DirectedGraph for FullConstraintGraph<'s> {
    type Node = RegionVid;
}

impl<'s> graph::WithNumNodes for FullConstraintGraph<'s> {
    fn num_nodes(&self) -> usize {
        self.first_constraints.len()
    }
}

impl<'s> graph::WithSuccessors for FullConstraintGraph<'s> {
    fn successors<'graph>(
        &'graph self,
        node: Self::Node,
    ) -> <Self as graph::GraphSuccessors<'graph>>::Iter {
        self.sub_regions(node)
    }
}

impl<'s, 'graph> graph::GraphSuccessors<'graph> for FullConstraintGraph<'s> {
    type Item = RegionVid;
    type Iter = Successors<'graph>;
}

