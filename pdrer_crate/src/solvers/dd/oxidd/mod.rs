// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod oxidd_bcdd;
pub mod oxidd_bdd;
pub mod oxidd_zbdd;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************

pub use oxidd_bcdd::OxiddBcdd;
pub use oxidd_bdd::OxiddBdd;
pub use oxidd_zbdd::OxiddZbdd;
