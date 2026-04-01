use super::super::controller::*;

///
/// Controller with a factory method
///
pub trait CreateableController : Controller {
    ///
    /// Creates a new instance of this controller
    ///
    fn new() -> Self;
}
