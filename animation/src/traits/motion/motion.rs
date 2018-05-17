use super::translate::*;

///
/// Describes ways in which a vector element can be moved and transformed over time.
/// Every element can have more than one motion attached to it, but for any given
/// element, each motion must appear only once.
/// 
#[derive(Clone, PartialEq, Debug)]
pub enum Motion {
    /// Describes how an element is translated over time
    Translate(TranslateMotion)
}

///
/// The possible types of motion
/// 
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MotionType {
    Translate
}