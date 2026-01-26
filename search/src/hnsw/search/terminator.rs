use sal_core::error::Error;

use crate::domain::Eval;

///
/// Do nothing, just terminates Eval sequence
pub struct Terminator {
}
//
impl Terminator {
    ///
    /// Returns [Terminator] new instance
    pub fn new() -> Self {
        Self { }
    }
}
//
impl Eval<(), Result<(), Error>> for Terminator {
    fn eval(&self, _: ()) -> Result<(), Error> {
        Ok(())
    }
}