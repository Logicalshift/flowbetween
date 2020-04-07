///
/// Possible options to use when converting an element to a path
///
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PathConversion {
	/// Use the fastest possible way to generate the path
	Fastest,

	/// Generate the path and then remove any points that are inside the path, so the path is suited for future
	/// arithmetic operations
	///
	/// (This is a function provided by `flo_curves`)
	RemoveInteriorPoints
}
