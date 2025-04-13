use std::cell::RefCell;
use std::rc::Rc;

use cudd_sys::cudd::Cudd_ReorderingType::CUDD_REORDER_SAME;
use cudd_sys::cudd::*;
use cudd_sys::DdManager;
use cudd_sys::DdNode;

use crate::solvers::dd::DDError;
use crate::solvers::dd::DecisionDiagramManager;

// ************************************************************************************************
// helper functions
// ************************************************************************************************

fn get(c: &Rc<RefCell<*mut DdManager>>) -> *mut DdManager {
    *c.borrow()
}

fn clear(c: &Rc<RefCell<*mut DdManager>>) {
    *c.borrow_mut() = std::ptr::null_mut();
}

fn was_cleared(c: &Rc<RefCell<*mut DdManager>>) -> bool {
    c.borrow().is_null()
}

// ************************************************************************************************
// Cudd Type
// ************************************************************************************************

pub(super) const CUDD_BDD: char = 'b';
pub(super) const CUDD_ZDD: char = 'z';

// ************************************************************************************************
// Node
// ************************************************************************************************

pub struct CuddBaseNode<const T: char> {
    m: Rc<RefCell<*mut DdManager>>,
    n: *mut DdNode,
}

#[allow(unsafe_code)]
impl<const T: char> CuddBaseNode<T> {
    fn new(m: Rc<RefCell<*mut DdManager>>, n: *mut DdNode) -> Self {
        unsafe {
            Cudd_Ref(n);
        }
        Self { m, n }
    }

    fn new_projection(m: Rc<RefCell<*mut DdManager>>, n: *mut DdNode) -> Self {
        Self { m, n }
    }
}

#[allow(unsafe_code)]
impl<const T: char> Drop for CuddBaseNode<T> {
    fn drop(&mut self) {
        if !was_cleared(&self.m) {
            match T {
                CUDD_BDD => unsafe {
                    Cudd_RecursiveDeref(get(&self.m), self.n);
                },
                CUDD_ZDD => unsafe {
                    Cudd_RecursiveDerefZdd(get(&self.m), self.n);
                },
                _ => unreachable!(),
            }
        }
    }
}

impl<const T: char> Clone for CuddBaseNode<T> {
    fn clone(&self) -> Self {
        Self::new(self.m.clone(), self.n)
    }
}

// ************************************************************************************************
// CuddBdd
// ************************************************************************************************

pub struct CuddBase<const T: char> {
    m: Rc<RefCell<*mut DdManager>>,
    vars: Vec<CuddBaseNode<T>>,
}

// ************************************************************************************************
// impl CuddBdd
// ************************************************************************************************

impl<const T: char> CuddBase<T> {
    fn check_validity(&self, f: &CuddBaseNode<T>) -> Result<(), DDError> {
        if f.m != self.m {
            return Err(DDError::DifferentManagers);
        }
        if was_cleared(&f.m) {
            return Err(DDError::ManagerAlreadyDeallocated);
        }
        Ok(())
    }

    fn check_memory(&self, n: *mut DdNode) -> Result<(), DDError> {
        if n.is_null() {
            return Err(DDError::OutOfMemory);
        }
        Ok(())
    }
}

#[allow(unsafe_code)]
impl<const T: char> Drop for CuddBase<T> {
    fn drop(&mut self) {
        unsafe { Cudd_Quit(get(&self.m)) };
        clear(&self.m);
    }
}

// ************************************************************************************************
// impl DecisionDiagramManager for CuddBdd
// ************************************************************************************************

#[allow(unsafe_code)]
impl<const T: char> DecisionDiagramManager for CuddBase<T> {
    type DecisionDiagram = CuddBaseNode<T>;

    fn new(number_of_vars: usize, number_of_threads: usize, max_memory_in_mb: usize) -> Self {
        assert!(
            number_of_threads == 1,
            "Cudd does not support multithreading."
        );

        let x = match T {
            CUDD_BDD => unsafe {
                Cudd_Init(
                    number_of_vars as u32,
                    0,
                    CUDD_UNIQUE_SLOTS,
                    CUDD_CACHE_SLOTS,
                    max_memory_in_mb * (1 << 20),
                )
            },
            CUDD_ZDD => unsafe {
                Cudd_Init(
                    0,
                    number_of_vars as u32,
                    CUDD_UNIQUE_SLOTS,
                    CUDD_CACHE_SLOTS,
                    max_memory_in_mb * (1 << 20),
                )
            },
            _ => unreachable!(),
        };
        let m = Rc::new(RefCell::new(x));

        // Get variables
        let mut vars = Vec::with_capacity(number_of_vars);
        for i in 0..number_of_vars {
            let n = match T {
                CUDD_BDD => unsafe { Cudd_bddIthVar(get(&m), i as i32) },
                CUDD_ZDD => unsafe { Cudd_zddIthVar(get(&m), i as i32) },
                _ => unreachable!(),
            };
            let var = CuddBaseNode::new_projection(m.clone(), n);
            vars.push(var);
        }

        match T {
            CUDD_BDD => unsafe {
                Cudd_AutodynEnable(get(&m), CUDD_REORDER_SAME);
            },
            CUDD_ZDD => unsafe {
                Cudd_AutodynEnableZdd(get(&m), CUDD_REORDER_SAME);
            },
            _ => unreachable!(),
        };

        Self { m, vars }
    }

    fn top(&mut self) -> Result<Self::DecisionDiagram, DDError> {
        let n = match T {
            CUDD_BDD => unsafe { Cudd_ReadOne(get(&self.m)) },
            CUDD_ZDD => unsafe { Cudd_ReadZddOne(get(&self.m), 0) },
            _ => unreachable!(),
        };
        Ok(CuddBaseNode::new(self.m.clone(), n))
    }

    fn bot(&mut self) -> Result<Self::DecisionDiagram, DDError> {
        let n = match T {
            CUDD_BDD => unsafe { Cudd_ReadLogicZero(get(&self.m)) },
            CUDD_ZDD => unsafe { Cudd_ReadZero(get(&self.m)) },
            _ => unreachable!(),
        };
        Ok(CuddBaseNode::new(self.m.clone(), n))
    }

    fn ithvar(&mut self, i: usize) -> Result<Self::DecisionDiagram, DDError> {
        match self.vars.get(i) {
            Some(x) => Ok(x.clone()),
            None => Err(DDError::OutOfBounds),
        }
    }

    fn apply_not(&mut self, f: &Self::DecisionDiagram) -> Result<Self::DecisionDiagram, DDError> {
        match T {
            CUDD_BDD => {
                self.check_validity(f)?;
                let n = unsafe { Cudd_Not(f.n) };
                self.check_memory(n)?;
                Ok(CuddBaseNode::new(self.m.clone(), n))
            }
            CUDD_ZDD => {
                let t = self.top()?;
                self.apply_diff(&t, f)
            }
            _ => unreachable!(),
        }
    }

    fn apply_and(
        &mut self,
        f: &Self::DecisionDiagram,
        g: &Self::DecisionDiagram,
    ) -> Result<Self::DecisionDiagram, DDError> {
        self.check_validity(f)?;
        self.check_validity(g)?;
        let n = match T {
            CUDD_BDD => unsafe { Cudd_bddAnd(get(&self.m), f.n, g.n) },
            CUDD_ZDD => unsafe { Cudd_zddIntersect(get(&self.m), f.n, g.n) },
            _ => unreachable!(),
        };
        self.check_memory(n)?;
        Ok(CuddBaseNode::new(self.m.clone(), n))
    }

    fn apply_or(
        &mut self,
        f: &Self::DecisionDiagram,
        g: &Self::DecisionDiagram,
    ) -> Result<Self::DecisionDiagram, DDError> {
        self.check_validity(f)?;
        self.check_validity(g)?;

        let n = match T {
            CUDD_BDD => unsafe { Cudd_bddOr(get(&self.m), f.n, g.n) },
            CUDD_ZDD => unsafe { Cudd_zddUnion(get(&self.m), f.n, g.n) },
            _ => unreachable!(),
        };

        self.check_memory(n)?;

        Ok(CuddBaseNode::new(self.m.clone(), n))
    }

    fn apply_diff(
        &mut self,
        f: &Self::DecisionDiagram,
        g: &Self::DecisionDiagram,
    ) -> Result<Self::DecisionDiagram, DDError> {
        match T {
            CUDD_BDD => {
                let not_g = self.apply_not(g)?;
                self.apply_and(f, &not_g)
            }
            CUDD_ZDD => {
                self.check_validity(f)?;
                self.check_validity(g)?;
                let n = unsafe { Cudd_zddDiff(get(&self.m), f.n, g.n) };
                self.check_memory(n)?;
                Ok(CuddBaseNode::new(self.m.clone(), n))
            }
            _ => unreachable!(),
        }
    }

    fn apply_imp(
        &mut self,
        f: &Self::DecisionDiagram,
        g: &Self::DecisionDiagram,
    ) -> Result<Self::DecisionDiagram, DDError> {
        let t = self.top()?;
        self.apply_ite(f, g, &t)
    }

    fn apply_xor(
        &mut self,
        f: &Self::DecisionDiagram,
        g: &Self::DecisionDiagram,
    ) -> Result<Self::DecisionDiagram, DDError> {
        match T {
            CUDD_BDD => {
                self.check_validity(f)?;
                self.check_validity(g)?;
                let n = unsafe { Cudd_bddXor(get(&self.m), f.n, g.n) };
                self.check_memory(n)?;
                Ok(CuddBaseNode::new(self.m.clone(), n))
            }
            CUDD_ZDD => {
                let f_or_g = self.apply_or(f, g)?;
                let f_and_g = self.apply_and(f, g)?;
                let n = self.apply_diff(&f_or_g, &f_and_g)?;
                Ok(n)
            }
            _ => unreachable!(),
        }
    }

    fn apply_xnor(
        &mut self,
        f: &Self::DecisionDiagram,
        g: &Self::DecisionDiagram,
    ) -> Result<Self::DecisionDiagram, DDError> {
        match T {
            CUDD_BDD => {
                self.check_validity(f)?;
                self.check_validity(g)?;
                let n = unsafe { Cudd_bddXnor(get(&self.m), f.n, g.n) };
                self.check_memory(n)?;
                Ok(CuddBaseNode::new(self.m.clone(), n))
            }
            CUDD_ZDD => {
                let f_xor_g = self.apply_xor(f, g)?;
                let n = self.apply_not(&f_xor_g)?;
                Ok(n)
            }
            _ => unreachable!(),
        }
    }

    fn apply_ite(
        &mut self,
        i: &Self::DecisionDiagram,
        t: &Self::DecisionDiagram,
        e: &Self::DecisionDiagram,
    ) -> Result<Self::DecisionDiagram, DDError> {
        self.check_validity(i)?;
        self.check_validity(t)?;
        self.check_validity(e)?;
        let n = match T {
            CUDD_BDD => unsafe { Cudd_bddIte(get(&self.m), i.n, t.n, e.n) },
            CUDD_ZDD => unsafe { Cudd_zddIte(get(&self.m), i.n, t.n, e.n) },
            _ => unreachable!(),
        };
        self.check_memory(n)?;
        Ok(CuddBaseNode::new(self.m.clone(), n))
    }

    fn iter_vars(
        &mut self,
    ) -> Result<impl ExactSizeIterator<Item = Self::DecisionDiagram> + DoubleEndedIterator, DDError>
    {
        Ok(self.vars.iter().cloned())
    }

    fn is_tautology(&mut self, f: &Self::DecisionDiagram) -> Result<bool, DDError> {
        self.check_validity(f)?;
        let t = self.top()?;
        self.are_equal(&t, f)
    }

    fn is_contradiction(&mut self, f: &Self::DecisionDiagram) -> Result<bool, DDError> {
        self.check_validity(f)?;
        let fal = self.bot()?;
        self.are_equal(&fal, f)
    }

    fn are_equal(
        &mut self,
        f: &Self::DecisionDiagram,
        g: &Self::DecisionDiagram,
    ) -> Result<bool, DDError> {
        self.check_validity(f)?;
        self.check_validity(g)?;
        Ok(f.n == g.n)
    }

    fn nodecount(&mut self, _: &Self::DecisionDiagram) -> Result<usize, DDError> {
        Err(DDError::ActionNotSupported)
    }

    fn allocated_nodes(&mut self) -> Result<usize, DDError> {
        let a = unsafe { Cudd_ReadKeys(get(&self.m)) };
        Ok(a as usize)
    }
}
