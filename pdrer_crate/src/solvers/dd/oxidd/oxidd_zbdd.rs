use super::oxidd_bdd::get_oxidd_capacities;
use crate::solvers::dd::DDError;
use crate::solvers::dd::DecisionDiagramManager;
use oxidd::zbdd::ZBDDFunction;
use oxidd::zbdd::ZBDDManagerRef;
use oxidd::BooleanFunction;
use oxidd::BooleanVecSet;
use oxidd::Function;
use oxidd::Manager;
use oxidd::ManagerRef;

pub struct OxiddZbdd {
    manager: ZBDDManagerRef,
    vars: Vec<ZBDDFunction>,
}

impl DecisionDiagramManager for OxiddZbdd {
    type DecisionDiagram = ZBDDFunction;

    fn new(number_of_vars: usize, number_of_threads: usize, max_memory_in_mb: usize) -> Self {
        let (x, y) = get_oxidd_capacities(max_memory_in_mb);
        let manager_ref = oxidd::zbdd::new_manager(x, y, number_of_threads as u32);
        let n = number_of_vars;
        let singletons: Vec<ZBDDFunction> = manager_ref.with_manager_exclusive(|manager| {
            (0..n)
                .map(|_| ZBDDFunction::new_singleton(manager).unwrap())
                .collect()
        });
        let vars = manager_ref.with_manager_shared(|manager| {
            singletons
                .iter()
                .map(|s| {
                    ZBDDFunction::from_edge(
                        manager,
                        oxidd::zbdd::var_boolean_function(manager, s.as_edge(manager)).unwrap(),
                    )
                })
                .collect()
        });

        OxiddZbdd {
            manager: manager_ref,
            vars,
        }
    }

    fn top(&mut self) -> Result<Self::DecisionDiagram, DDError> {
        let x = self
            .manager
            .with_manager_shared(|manager| ZBDDFunction::t(manager));
        Ok(x)
    }

    fn bot(&mut self) -> Result<Self::DecisionDiagram, DDError> {
        let x = self
            .manager
            .with_manager_shared(|manager| ZBDDFunction::f(manager));
        Ok(x)
    }

    fn ithvar(&mut self, i: usize) -> Result<Self::DecisionDiagram, DDError> {
        match self.vars.get(i) {
            Some(x) => Ok(x.clone()),
            None => Err(DDError::OutOfBounds),
        }
    }

    fn apply_not(&mut self, f: &Self::DecisionDiagram) -> Result<Self::DecisionDiagram, DDError> {
        match f.not() {
            Ok(x) => Ok(x),
            Err(_) => Err(DDError::OutOfMemory),
        }
    }

    fn apply_and(
        &mut self,
        f: &Self::DecisionDiagram,
        g: &Self::DecisionDiagram,
    ) -> Result<Self::DecisionDiagram, DDError> {
        match f.and(g) {
            Ok(x) => Ok(x),
            Err(_) => Err(DDError::OutOfMemory),
        }
    }

    fn apply_or(
        &mut self,
        f: &Self::DecisionDiagram,
        g: &Self::DecisionDiagram,
    ) -> Result<Self::DecisionDiagram, DDError> {
        match f.or(g) {
            Ok(x) => Ok(x),
            Err(_) => Err(DDError::OutOfMemory),
        }
    }

    fn apply_diff(
        &mut self,
        f: &Self::DecisionDiagram,
        g: &Self::DecisionDiagram,
    ) -> Result<Self::DecisionDiagram, DDError> {
        match g.imp_strict(f) {
            Ok(x) => Ok(x),
            Err(_) => Err(DDError::OutOfMemory),
        }
    }

    fn apply_imp(
        &mut self,
        f: &Self::DecisionDiagram,
        g: &Self::DecisionDiagram,
    ) -> Result<Self::DecisionDiagram, DDError> {
        match f.imp(g) {
            Ok(x) => Ok(x),
            Err(_) => Err(DDError::OutOfMemory),
        }
    }

    fn apply_xor(
        &mut self,
        f: &Self::DecisionDiagram,
        g: &Self::DecisionDiagram,
    ) -> Result<Self::DecisionDiagram, DDError> {
        match f.xor(g) {
            Ok(x) => Ok(x),
            Err(_) => Err(DDError::OutOfMemory),
        }
    }

    fn apply_xnor(
        &mut self,
        f: &Self::DecisionDiagram,
        g: &Self::DecisionDiagram,
    ) -> Result<Self::DecisionDiagram, DDError> {
        match f.equiv(g) {
            Ok(x) => Ok(x),
            Err(_) => Err(DDError::OutOfMemory),
        }
    }

    fn apply_ite(
        &mut self,
        i: &Self::DecisionDiagram,
        t: &Self::DecisionDiagram,
        e: &Self::DecisionDiagram,
    ) -> Result<Self::DecisionDiagram, DDError> {
        match i.ite(t, e) {
            Ok(x) => Ok(x),
            Err(_) => Err(DDError::OutOfMemory),
        }
    }

    fn iter_vars(
        &mut self,
    ) -> Result<impl ExactSizeIterator<Item = Self::DecisionDiagram> + DoubleEndedIterator, DDError>
    {
        Ok(self.vars.iter().cloned())
    }

    fn is_tautology(&mut self, f: &Self::DecisionDiagram) -> Result<bool, DDError> {
        Ok(f.valid())
    }

    fn is_contradiction(&mut self, f: &Self::DecisionDiagram) -> Result<bool, DDError> {
        Ok(!f.satisfiable())
    }

    fn are_equal(
        &mut self,
        f: &Self::DecisionDiagram,
        g: &Self::DecisionDiagram,
    ) -> Result<bool, DDError> {
        Ok(f == g)
    }

    fn nodecount(&mut self, f: &Self::DecisionDiagram) -> Result<usize, DDError> {
        Ok(f.node_count())
    }

    fn allocated_nodes(&mut self) -> Result<usize, DDError> {
        Ok(self
            .manager
            .with_manager_shared(|manager| manager.num_inner_nodes()))
    }
}
