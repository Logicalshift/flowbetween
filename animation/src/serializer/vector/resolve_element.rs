use super::super::super::traits::*;

///
/// Trait implemented by structures that need to have external elements resolved
///
pub trait ResolveElements<T> {
    ///
    /// Creates the object that this resolves to, given a function that can look up
    /// elements by ID. 
    ///
    fn resolve(self, find_element: &mut dyn FnMut(ElementId) -> Option<Vector>) -> Option<T>;
}

///
/// Basic implementation of the resolve elements trait that resolves via a callback to a closure
///
pub (crate) struct ElementResolver<TFn: Sized, T>(pub TFn)
where TFn: FnOnce(&mut dyn FnMut(ElementId) -> Option<Vector>) -> Option<T>;

impl<TFn, T> ResolveElements<T> for ElementResolver<TFn, T>
where TFn: FnOnce(&mut dyn FnMut(ElementId) -> Option<Vector>) -> Option<T> {
    fn resolve(self, find_element: &mut dyn FnMut(ElementId) -> Option<Vector>) -> Option<T> {
        let ElementResolver(resolve) = self;
        resolve(find_element)
    }
}
