///
/// Options that control the output of a fill operation
///
#[derive(Clone, Debug, PartialEq)]
pub enum FillOption {
    /// The distance between rays to use when finding points on the fill
    RayCastDistance(f64),

    /// The minimum gap size that the fill can 'escape' through
    MinGap(f64),

    /// Set to true if the fill should appear behind any element that a ray hits
    FillBehind,

    /// The fill will only perform a single pass to find collisions (it won't reach around corners)
    Convex,

    /// The fill will perform extra passes if there are two points that are far apart so that it fills the maximum space possible (it will reach around corners)
    Concave,
}
