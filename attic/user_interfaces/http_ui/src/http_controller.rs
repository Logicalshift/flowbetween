use ui::*;

///
/// Trait implemented by controllers types that can be used as the target of a HTTP
/// session
///
pub trait HttpController : Controller {
    ///
    /// Creates a new instance of this controller
    ///
    fn start_new() -> Self;
}
