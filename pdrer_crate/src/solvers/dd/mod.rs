//! module that holds objects and traits that performing decision diagrams in various ways.

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod cudd;
pub mod oxidd;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************

pub use cudd::CuddBdd;
pub use cudd::CuddZdd;
pub use oxidd::OxiddBcdd;
pub use oxidd::OxiddBdd;
pub use oxidd::OxiddZbdd;

// ************************************************************************************************
// Sat Solver trait
// ************************************************************************************************

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DDError {
    OutOfMemory,
    OutOfBounds,
    DifferentManagers,
    ManagerAlreadyDeallocated,
    InvalidInput,
    ActionNotSupported,
}

pub trait DecisionDiagramManager {
    /// Type that holds the decision diagram, decision diagram should have a clone implementation
    /// because we can always negate twice to clone anyway. So all BDD implementations should have
    /// be able to clone.
    type DecisionDiagram: Clone;

    /// Initialize a decision diagram manager with the provided number of variables
    fn new(number_of_vars: usize, number_of_threads: usize, max_memory_in_mb: usize) -> Self;

    /// Get the decision diagram that represents true
    fn top(&mut self) -> Result<Self::DecisionDiagram, DDError>;

    /// Get the decision diagram that represents false
    fn bot(&mut self) -> Result<Self::DecisionDiagram, DDError>;

    /// Get the decision diagram that represents the i-th variable
    fn ithvar(&mut self, i: usize) -> Result<Self::DecisionDiagram, DDError>;

    /// Get the decision diagram that represents the not operation on the DD.
    fn apply_not(&mut self, f: &Self::DecisionDiagram) -> Result<Self::DecisionDiagram, DDError>;

    /// Get the decision diagram that represents the and operation between two DDs.
    fn apply_and(
        &mut self,
        f: &Self::DecisionDiagram,
        g: &Self::DecisionDiagram,
    ) -> Result<Self::DecisionDiagram, DDError>;

    /// Get the decision diagram that represents the or operation between two DDs.
    fn apply_or(
        &mut self,
        f: &Self::DecisionDiagram,
        g: &Self::DecisionDiagram,
    ) -> Result<Self::DecisionDiagram, DDError>;

    /// Get the decision diagram that represents the diff operation between two DDs.
    fn apply_diff(
        &mut self,
        f: &Self::DecisionDiagram,
        g: &Self::DecisionDiagram,
    ) -> Result<Self::DecisionDiagram, DDError>;

    /// Get the decision diagram that represents the implication operation between two DDs.
    fn apply_imp(
        &mut self,
        f: &Self::DecisionDiagram,
        g: &Self::DecisionDiagram,
    ) -> Result<Self::DecisionDiagram, DDError>;

    /// Get the decision diagram that represents the xor operation between two DDs.
    fn apply_xor(
        &mut self,
        f: &Self::DecisionDiagram,
        g: &Self::DecisionDiagram,
    ) -> Result<Self::DecisionDiagram, DDError>;

    /// Get the decision diagram that represents the xnor operation (A.K.A. equivalence) between two DDs.
    fn apply_xnor(
        &mut self,
        f: &Self::DecisionDiagram,
        g: &Self::DecisionDiagram,
    ) -> Result<Self::DecisionDiagram, DDError>;

    /// Get the decision diagram that represents the `if then else` operation between two DDs.
    fn apply_ite(
        &mut self,
        i: &Self::DecisionDiagram,
        t: &Self::DecisionDiagram,
        e: &Self::DecisionDiagram,
    ) -> Result<Self::DecisionDiagram, DDError>;

    /// Iterate over the DDs that represents the state variables.
    fn iter_vars(
        &mut self,
    ) -> Result<impl ExactSizeIterator<Item = Self::DecisionDiagram> + DoubleEndedIterator, DDError>;

    // /// Compute the existential quantification over given variable
    // fn exists(&mut self, f: &Self::DecisionDiagram, i: usize) -> Result<bool, DDError>;

    /// Check if the provided decision diagram is always true
    fn is_tautology(&mut self, f: &Self::DecisionDiagram) -> Result<bool, DDError>;

    /// Check if the provided decision diagram is always false
    fn is_contradiction(&mut self, f: &Self::DecisionDiagram) -> Result<bool, DDError>;

    /// Check if the two decision diagrams are equivalent
    fn are_equal(
        &mut self,
        f: &Self::DecisionDiagram,
        g: &Self::DecisionDiagram,
    ) -> Result<bool, DDError>;

    /// Get the number of nodes in the DD.
    fn nodecount(&mut self, f: &Self::DecisionDiagram) -> Result<usize, DDError>;

    /// Get the number of total allocated nodes in the Manager.
    fn allocated_nodes(&mut self) -> Result<usize, DDError>;
}
