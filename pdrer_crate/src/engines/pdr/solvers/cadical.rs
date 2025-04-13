use crate::{formulas::Literal, solvers::sat::incremental::CaDiCalSolver};

use super::PropertyDirectedReachabilitySolver;

impl PropertyDirectedReachabilitySolver for CaDiCalSolver {
    fn new(seed: u64) -> Self {
        Self::new(seed)
    }

    // fn reserve_variables(&mut self, max_var: crate::formulas::Variable) {
    //     self.reserve_variables(max_var);
    // }

    fn add_clause<I>(&mut self, clause: I)
    where
        I: IntoIterator<Item = Literal>,
    {
        self.add_clause(clause)
    }

    fn solve<I, U>(
        &mut self,
        assumptions: I,
        constraint_clause: U,
    ) -> crate::solvers::sat::incremental::SatResult
    where
        I: IntoIterator<Item = Literal>,
        U: IntoIterator<Item = Literal>,
    {
        self.solve(assumptions, constraint_clause)
    }

    fn val(&mut self, lit: Literal) -> Option<bool> {
        self.val(lit)
    }

    fn failed(&mut self, lit: Literal) -> bool {
        self.failed(lit)
    }

    // fn constraint_failed(&mut self) -> bool {
    //     self.constraint_failed()
    // }

    // fn simplify(&mut self) -> Option<SatResult> {
    //     self.simplify(3)
    // }
}
