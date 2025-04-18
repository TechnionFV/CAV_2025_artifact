// ************************************************************************************************
// use
// ************************************************************************************************

use super::Utils;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Utils {
    pub fn ensure(cond: bool, msg: &str) -> Result<(), String> {
        if cond {
            Ok(())
        } else {
            Err(msg.to_owned())
        }
    }
}
