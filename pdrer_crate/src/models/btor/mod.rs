//! object for reading .btor files that adhere to the BTOR2 format.
//! For more information on the BTOR2 format, see <https://fmv.jku.at/cav18-btor2/>

// ************************************************************************************************
// use
// ************************************************************************************************

use self::line::BtorLine;
use super::UniqueSortedHashMap;
use gates::BtorNode;

// ************************************************************************************************
// struct
// ************************************************************************************************

pub struct BTOR {
    _nodes: UniqueSortedHashMap<BtorLine, BtorNode>, /* [0..maxvar] */
}

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod construction;
pub mod gates;
pub mod line;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************
