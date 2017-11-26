use std::marker::PhantomData;

///
/// An empty iterator, used as a placeholder when there are no items available
/// 
pub struct EmptyIterator<Item> {
    phantom: PhantomData<Item>
}

impl<Item> EmptyIterator<Item> {
    pub fn new() -> EmptyIterator<Item> {
        EmptyIterator { phantom: PhantomData }
    }
}

impl<IteratorItem> Iterator for EmptyIterator<IteratorItem> {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<IteratorItem> {
        None
    }
}
