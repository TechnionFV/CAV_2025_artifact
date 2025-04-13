use std::fmt::Debug;

use crate::{
    formulas::{Clause, Variable},
    models::UniqueSortedVec,
    solvers::dd::DecisionDiagramManager,
};

pub struct DeltaElement<D: DecisionDiagramManager> {
    clause: Clause,
    dd: Option<D::DecisionDiagram>,
    coi: UniqueSortedVec<Variable>,
    state_vars_in_coi: UniqueSortedVec<Variable>,
    vars_without_ev: UniqueSortedVec<Variable>,
    // fractions_that_are_already_propagated: Vec<Clause>,
}

impl<D: DecisionDiagramManager> DeltaElement<D> {
    pub fn new(
        clause: Clause,
        dd: Option<D::DecisionDiagram>,
        coi: UniqueSortedVec<Variable>,
        state_vars_in_coi: UniqueSortedVec<Variable>,
        vars_without_ev: UniqueSortedVec<Variable>,
    ) -> Self {
        Self {
            clause,
            dd,
            coi,
            state_vars_in_coi,
            vars_without_ev,
            // fractions_that_are_already_propagated: vec![],
        }
    }

    pub fn clause(&self) -> &Clause {
        &self.clause
    }

    pub fn dd(&self) -> &Option<D::DecisionDiagram> {
        &self.dd
    }

    pub fn state_vars_in_coi(&self) -> &UniqueSortedVec<Variable> {
        &self.state_vars_in_coi
    }

    pub fn vars_without_ev(&self) -> &UniqueSortedVec<Variable> {
        &self.vars_without_ev
    }

    pub fn coi(&self) -> &UniqueSortedVec<Variable> {
        &self.coi
    }

    // pub fn bva_clause(&self) -> &Option<Clause> {
    //     &self.bva_clause
    // }

    pub fn unpack_clause(self) -> Clause {
        self.clause
    }

    // pub fn was_fraction_already_propagated(&self, fraction: &Clause) -> bool {
    //     self.fractions_that_are_already_propagated
    //         .contains(fraction)
    // }

    // pub fn add_propagated_fraction(&mut self, fraction: Clause) {
    //     self.fractions_that_are_already_propagated.push(fraction);
    // }
}

impl<D: DecisionDiagramManager> Debug for DeltaElement<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.clause)
    }
}

impl<D: DecisionDiagramManager> Clone for DeltaElement<D> {
    fn clone(&self) -> Self {
        Self {
            clause: self.clause.clone(),
            dd: self.dd.clone(),
            coi: self.coi.clone(),
            state_vars_in_coi: self.state_vars_in_coi.clone(),
            vars_without_ev: self.vars_without_ev.clone(),
            // fractions_that_are_already_propagated: self
            //     .fractions_that_are_already_propagated
            //     .clone(),
        }
    }
}
