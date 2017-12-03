use std::ops::Deref;
use std::marker::PhantomData;

///
/// Given something that can be derefed, maps it to something else
/// 
/// This can be used to provide a view of the internals of a lock without needing
/// to supply the actual locked item, for example. This is useful for avoiding
/// the need to clone things out of a lock in order to iterate over them or
/// access them.
/// 
pub struct DerefMap<TSource, TSourceDeref, TTarget, MapFn>
where TSource: Deref<Target=TSourceDeref>, MapFn: Fn(&TSourceDeref) -> &TTarget {
    /// The original Deref object
    source: TSource,

    /// A function that maps the value that's derefed to the new target type
    map: MapFn,

    /// Phantom data for the target
    phantom: PhantomData<TTarget>,
}

impl<TSource, TSourceDeref, TTarget, MapFn> DerefMap<TSource, TSourceDeref, TTarget, MapFn>
where TSource: Deref<Target=TSourceDeref>, MapFn: Fn(&TSourceDeref) -> &TTarget {
    ///
    /// Creates a new Deref map
    /// 
    pub fn map(source: TSource, map: MapFn) -> DerefMap<TSource, TSourceDeref, TTarget, MapFn> {
        DerefMap {
            source:     source,
            map:        map,
            phantom:    PhantomData
        }
    }
}

impl<TSource, TSourceDeref, TTarget, MapFn> Deref for DerefMap<TSource, TSourceDeref, TTarget, MapFn>
where TSource: Deref<Target=TSourceDeref>, MapFn: Fn(&TSourceDeref) -> &TTarget {
    type Target = TTarget;

    fn deref(&self) -> &TTarget {
        (self.map)(self.source.deref())
    }
}
