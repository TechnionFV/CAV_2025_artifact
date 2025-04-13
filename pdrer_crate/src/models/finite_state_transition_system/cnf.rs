// ************************************************************************************************
// use
// ************************************************************************************************

use super::FiniteStateTransitionSystem;
use crate::formulas::CNF;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl FiniteStateTransitionSystem {
    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn construct_property_cnf(&self, negate_property: bool, use_internal_signals: bool) -> CNF {
        // first express bad in terms of state variables
        let mut bad = self.get_property_connector().to_owned();
        if negate_property {
            bad.add_clause(!self.get_property_on_internals().to_owned());
        } else {
            bad.append(self.get_property_on_internals().to_cnf());
        }

        // then express the invariant in terms of state variables
        bad.append(self.get_invariant_constraints_connector().to_owned());
        bad.append(self.get_invariant_constraints_on_internals().to_cnf());

        // if internal signals will be used in the proof then add the transition relation connector
        if use_internal_signals {
            bad.append(self.get_transition_connector().to_owned());
        }
        bad
    }

    pub fn construct_transition_cnf(
        &self,
        add_all_internal_signals_to_first_cycle: bool,
        add_all_internal_signals_to_second_cycle: bool,
        add_invariant_to_first_cycle: bool,
        add_invariant_to_second_cycle: bool,
    ) -> CNF {
        // first express transition in terms of state variables
        let mut transition = self.get_transition_connector().to_owned();
        transition.append(self.get_transition_on_internals().to_owned());

        // then add the invariant to the first clock cycle
        if add_invariant_to_first_cycle {
            transition.append(self.get_invariant_constraints_connector().to_owned());
            transition.append(self.get_invariant_constraints_on_internals().to_cnf());
        }

        if add_invariant_to_second_cycle {
            // then add the invariant to the second clock cycle
            transition.append({
                let mut a = self.get_invariant_constraints_connector().to_owned();
                a.append(self.get_invariant_constraints_on_internals().to_cnf());
                self.add_tags_to_relation(&mut a, 1);
                a
            });
        }

        // if internal signals will be used in the proof then add all the connectors
        if add_all_internal_signals_to_first_cycle {
            transition.append(self.get_property_connector().to_owned());
            transition.append(self.get_invariant_constraints_connector().to_owned());
        }

        if add_all_internal_signals_to_second_cycle {
            transition.append({
                let mut a = self.get_invariant_constraints_connector().to_owned();
                a.append(self.get_property_connector().to_owned());
                a.append(self.get_transition_connector().to_owned());
                self.add_tags_to_relation(&mut a, 1);
                a
            });
        }

        transition
    }

    pub fn construct_initial_cnf(&self, use_internal_signals: bool) -> CNF {
        // first express initial in terms of state variables
        let mut initial = self.get_initial_relation().to_cnf();

        // then express the invariant in terms of state variables
        initial.append(self.construct_invariant_constraint_cnf(use_internal_signals));

        initial
    }

    pub fn construct_invariant_constraint_cnf(&self, use_internal_signals: bool) -> CNF {
        let mut cnf = self.get_invariant_constraints_on_internals().to_cnf();

        if use_internal_signals {
            cnf.append(self.construct_cnf());
        } else if !cnf.is_empty() {
            cnf.append(self.get_invariant_constraints_connector().to_owned());
        }

        cnf
    }

    /// constructs the cnf that represents all internal signals that are used
    pub fn construct_cnf(&self) -> CNF {
        let mut cnf = self.get_transition_connector().to_owned();
        cnf.append(self.get_invariant_constraints_connector().to_owned());
        cnf.append(self.get_property_connector().to_owned());
        cnf
    }
}
