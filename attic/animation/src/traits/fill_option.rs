///
/// The algorithm to use when creating the fill path
///
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FillAlgorithm {
    /// The fill will only perform a single pass to find collisions (it won't reach around corners)
    Convex,

    /// The fill will perform extra passes if there are two points that are far apart so that it fills the maximum space possible (it will reach around corners)
    Concave,
}

///
/// Where the fill path should be created
///
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FillPosition {
    /// The fill should be created in front of the specified element
    InFront,

    /// The fill should be created behind the specified element
    Behind
}

///
/// Options that control the output of a fill operation
///
#[derive(Clone, PartialEq, Debug)]
pub enum FillOption {
    /// The distance between rays to use when finding points on the fill
    RayCastDistance(f64),

    /// The minimum gap size that the fill can 'escape' through
    MinGap(f64),

    /// The maximum fit error for the fill path that was traced out
    FitPrecision(f64),

    /// The algorithm to use to create this fill
    Algorithm(FillAlgorithm),

    /// Where to place the path that results from this fill
    Position(FillPosition)
}
