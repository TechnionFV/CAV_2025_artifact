use crate::solvers::dd::DDError;
use crate::solvers::dd::DecisionDiagramManager;
use oxidd::bdd::BDDFunction;
use oxidd::bdd::BDDManagerRef;
use oxidd::BooleanFunction;
use oxidd::Function;
use oxidd::Manager;
use oxidd::ManagerRef;

pub struct OxiddBdd {
    manager: BDDManagerRef,
    vars: Vec<BDDFunction>,
}

pub fn get_oxidd_capacities(max_memory_in_mb: usize) -> (usize, usize) {
    const BYTES_PER_NODE: usize = std::mem::size_of::<BDDFunction>();
    const BYTES_PER_CACHE_ENTRY: usize = 16;
    const CACHE_RATIO: usize = 64;
    const BYTES_PER_CACHE_UNIT: usize = BYTES_PER_NODE * CACHE_RATIO + BYTES_PER_CACHE_ENTRY;
    // Nodes = 1, 2, 3, 4, ..., 63, 64, 65, 66, ..., 127, 128
    //         |<-- 1st cache unit -->|<-- 2nd cache unit -->|
    let max_memory_in_bytes = max_memory_in_mb * (1 << 20);
    let number_of_cache_units = max_memory_in_bytes / BYTES_PER_CACHE_UNIT;
    let number_of_nodes = number_of_cache_units * CACHE_RATIO;
    (number_of_nodes, number_of_cache_units)
}

impl DecisionDiagramManager for OxiddBdd {
    type DecisionDiagram = BDDFunction;

    fn new(number_of_vars: usize, number_of_threads: usize, max_memory_in_mb: usize) -> Self {
        let (x, y) = get_oxidd_capacities(max_memory_in_mb);
        let manager_ref = oxidd::bdd::new_manager(x, y, number_of_threads as u32);
        let mut vars = Vec::with_capacity(number_of_vars);
        manager_ref.with_manager_exclusive(|manager| {
            for _ in 0..number_of_vars {
                let v = oxidd::bdd::BDDFunction::new_var(manager).unwrap();
                vars.push(v);
            }
        });

        OxiddBdd {
            manager: manager_ref,
            vars,
        }
    }

    fn top(&mut self) -> Result<Self::DecisionDiagram, DDError> {
        let x = self
            .manager
            .with_manager_shared(|manager| BDDFunction::t(manager));
        Ok(x)
    }

    fn bot(&mut self) -> Result<Self::DecisionDiagram, DDError> {
        let x = self
            .manager
            .with_manager_shared(|manager| BDDFunction::f(manager));
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
