// ************************************************************************************************
// use
// ************************************************************************************************

use super::Frames;
use crate::{
    engines::pdr::{
        definition_library::DefinitionLibrary, delta_element::DeltaElement,
        shared_objects::SharedObjects, PropertyDirectedReachabilitySolver,
    },
    formulas::Clause,
    function,
    models::{time_stats::function_timer::FunctionTimer, UniqueSortedVec},
    solvers::dd::DecisionDiagramManager,
};
use std::cmp::min;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<T: PropertyDirectedReachabilitySolver, D: DecisionDiagramManager> Frames<T, D> {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    pub(super) fn is_clause_redundant(&mut self, clause: &DeltaElement<D>, k: usize) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        // make sure that no higher frame already contains this clause
        for frame in self.frames.iter().skip(k) {
            for old_clause in frame.get_delta().iter() {
                if Self::does_a_imply_b(old_clause, clause, &mut self.definition_library, &self.s) {
                    return true;
                }
            }
        }
        false
    }

    /// returns the lowest frame that was updated, and a boolean indicating if the clause was already in the frame
    fn remove_subsumed_clauses(
        &mut self,
        clause: &DeltaElement<D>,
        start: usize,
        k: usize,
    ) -> usize {
        let mut lowest_frame = k;
        let mut to_remove = vec![];

        for (i, f) in self.frames.iter().enumerate().take(k + 1).skip(start) {
            for (x, de) in f.get_delta().iter().enumerate() {
                if Self::does_a_imply_b(clause, de, &mut self.definition_library, &self.s) {
                    lowest_frame = min(i, lowest_frame);
                    to_remove.push((i, x));
                }
            }
        }

        self.remove(&to_remove);

        lowest_frame
    }

    pub(super) fn mark_clause_added(
        &mut self,
        de: &DeltaElement<D>,
        k: usize,
        check_subsumption_from_same_frame_only: bool,
    ) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        let start = if check_subsumption_from_same_frame_only {
            if let Some((i, _)) = self.frames[k - 1]
                .get_delta()
                .iter()
                .enumerate()
                .find(|x| x.1.clause() == de.clause())
            {
                let to_remove = vec![(k - 1, i)];
                self.remove(&to_remove);
            }
            k
        } else {
            1
        };

        let existed = self.frames[k]
            .get_delta()
            .iter()
            .any(|x| x.clause() == de.clause());
        let lowest_frame = self.remove_subsumed_clauses(de, start, k);

        self.lowest_frame_that_was_updated_since_last_propagate = min(
            lowest_frame,
            self.lowest_frame_that_was_updated_since_last_propagate,
        );

        debug_assert!(!existed);
        if existed {
            return true;
        }

        for d in start..k {
            self.solvers.add_frame_clause(d, de.clause());
            self.frames[d].increment_hash();
        }

        false
    }

    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn does_clause_use_ev(d_lib: &DefinitionLibrary<T, D>, c: &Clause) -> bool {
        d_lib.is_extension_variable(c.max_variable())
    }

    pub fn make_delta_element(&mut self, clause: Clause) -> DeltaElement<D> {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());
        let ev = self.s.parameters.er;

        let dd = if ev {
            self.definition_library.clause_to_bdd_cached(&clause).ok()
        } else {
            None
        };

        let coi = if ev {
            self.definition_library
                .build_coi(clause.iter().map(|l| l.variable()))
        } else {
            UniqueSortedVec::from_ordered_set(vec![])
        };

        let state_vars_in_coi = if ev {
            let mut a = coi.clone();
            a.retain(|v| !self.definition_library.is_extension_variable(*v));
            a
        } else {
            UniqueSortedVec::from_ordered_set(vec![])
        };

        let vars_without_ev = if ev {
            UniqueSortedVec::from_ordered_set(
                clause
                    .iter()
                    .map(|l| l.variable())
                    .filter(|l| !self.definition_library.is_extension_variable(*l))
                    .collect(),
            )
        } else {
            UniqueSortedVec::new()
        };

        DeltaElement::new(clause, dd, coi, state_vars_in_coi, vars_without_ev)
    }

    fn check_using_bdds(
        a: &DeltaElement<D>,
        b: &DeltaElement<D>,
        d_lib: &mut DefinitionLibrary<T, D>,
    ) {
        debug_assert!(
            !d_lib.solve_implies(a, b),
            "Intersection check failed.\na = {}\nb = {}\ncoi_a = {}\ncoi_b = {}\nd_lib:\n{}",
            a.clause(),
            b.clause(),
            a.coi(),
            b.coi(),
            d_lib
        );
    }

    fn exists_state_var_in_a_not_in_cone_of_b(
        a: &DeltaElement<D>,
        b: &DeltaElement<D>,
        d_lib: &DefinitionLibrary<T, D>,
    ) -> bool {
        a.clause()
            .iter()
            .map(|v| v.variable())
            .filter(|v| !d_lib.is_extension_variable(*v))
            .any(|v| !b.coi().contains(&v))
    }

    fn exists_var_in_a_with_cone_that_does_not_intersect_cone_of_b(
        a: &DeltaElement<D>,
        b: &DeltaElement<D>,
        d_lib: &DefinitionLibrary<T, D>,
    ) -> bool {
        a.clause()
            .iter()
            .map(|v| v.variable())
            .filter(|v| d_lib.is_extension_variable(*v))
            .any(|v| d_lib.get_coi_of_variable(v).intersect(b.coi()).is_empty())
    }

    pub fn does_a_imply_b(
        a: &DeltaElement<D>,
        b: &DeltaElement<D>,
        d_lib: &mut DefinitionLibrary<T, D>,
        s: &SharedObjects,
    ) -> bool {
        let _timer = FunctionTimer::start(function!(), s.time_stats.clone());

        s.pdr_stats
            .borrow_mut()
            .increment_generic_count("does_a_imply_b total calls");

        if a.clause()
            .peek()
            .peek()
            .is_subset_of(b.clause().peek().peek())
        {
            s.pdr_stats
                .borrow_mut()
                .increment_generic_count("does_a_imply_b solved by subsumption (true)");
            return true;
        }

        if !s.parameters.er_impl {
            s.pdr_stats
                .borrow_mut()
                .increment_generic_count("does_a_imply_b syntactic only");
            return false;
        }

        if d_lib.is_empty() {
            s.pdr_stats
                .borrow_mut()
                .increment_generic_count("does_a_imply_b d_lib empty");
            return false;
        }

        if (!Self::does_clause_use_ev(d_lib, a.clause()))
            && (!Self::does_clause_use_ev(d_lib, b.clause()))
        {
            s.pdr_stats
                .borrow_mut()
                .increment_generic_count("does_a_imply_b neither clause contains EVs");
            Self::check_using_bdds(a, b, d_lib);
            return false;
        }

        // avoid building BDDs when the cone of influence of the two clauses do not intersect
        if Self::exists_state_var_in_a_not_in_cone_of_b(a, b, d_lib) {
            Self::check_using_bdds(a, b, d_lib);
            s.pdr_stats
                .borrow_mut()
                .increment_generic_count("does_a_imply_b COI subtraction");
            return false;
        }

        if Self::exists_var_in_a_with_cone_that_does_not_intersect_cone_of_b(a, b, d_lib) {
            Self::check_using_bdds(a, b, d_lib);
            s.pdr_stats
                .borrow_mut()
                .increment_generic_count("does_a_imply_b EV COI subtraction 2");
            return false;
        }

        d_lib.solve_implies(a, b)
    }

    pub fn insert_clause_to_highest_frame_possible(
        &mut self,
        clause: Clause,
        mut k: usize,
    ) -> usize {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        debug_assert!(!clause.is_empty());

        // let mut another_iteration = true;
        while k < self.depth() {
            if self.is_clause_guaranteed_after_transition_if_assumed(&clause, k) {
                k += 1;
            } else {
                break;
            }
        }

        let de = self.make_delta_element(clause);
        self.insert_clause_to_exact_frame(de, k, false);

        k
    }

    /// adds a clause to the frame that is required, also removes similar clauses from other frame.
    /// A list must be provided to indicate a history of clauses that this clause was derived from,
    /// this history is only relevant if using extension literals. (This history helps us know what
    /// clauses are implied by this new clause). The last clause in the list is the representative clause.
    ///
    pub fn insert_clause_to_exact_frame(
        &mut self,
        de: DeltaElement<D>,
        k: usize,
        is_propagated: bool,
    ) {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        debug_assert!(k > 0, "Trying to insert a clause into the initial frame.");
        // make clause canonical with definitions
        debug_assert_eq!(
            de.clause(),
            &self
                .definition_library
                .make_clause_canonical(de.clause().to_owned()),
            "Trying to insert a clause that is not canonical."
        );
        debug_assert!(
            !self.is_cube_initial(&!de.clause().to_owned()),
            "This clause is not satisfied by all initial states."
        );
        debug_assert!(
            {
                let i = if k == self.frames.len() - 1 { k } else { k - 1 };
                self.solvers
                    .is_clause_guaranteed_after_transition_if_assumed(i, de.clause())
            },
            "This clause has a predecessor in the previous frame."
        );

        self.mark_clause_added(&de, k, is_propagated);

        debug_assert!(
            !self.is_clause_redundant(&de, k),
            "Clause already in frame."
        );
        // debug_assert!(self.regression_check());
        self.solvers.add_frame_clause(k, de.clause());
        self.frames[k].push_to_delta_and_increment_hash(de);
        // debug_assert!(self.regression_check());
    }
}
