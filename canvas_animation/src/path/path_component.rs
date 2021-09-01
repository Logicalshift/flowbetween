use super::animation_path::*;

use flo_canvas::*;
use flo_curves::*;
use flo_curves::geo::*;
use flo_curves::bezier::path::*;

///
/// A path component represents a single closed path 
///
pub struct PathComponent<'a> {
    /// The slice of path operations for this path component. The first item in the slice is the move operation, and the last is the item prior to the next move operation
    slice: &'a [PathOp]
}

impl<'a> PathComponent<'a> {
    ///
    /// Divides an animation path into path components
    ///
    pub fn components_from_path(path: &'a AnimationPath) -> Vec<PathComponent<'a>> {
        // The result of this operation
        let mut path_components = vec![];

        // Short-circuit if the path is empty
        if path.path.len() == 0 {
            return path_components;
        }

        // The index of the 'Move' instruction that started the current path
        let mut initial_index   = 0;

        // Find the end component of each path
        for path_index in 1..path.path.len() {
            if let PathOp::Move(_, _) = &path.path[path_index] {
                // Split a component at this point
                let component = PathComponent { slice: &path.path[initial_index..path_index] };
                path_components.push(component);

                // This move instruction is the start of the next path
                initial_index = path_index;
            }
        }

        // The last component might not be broken off with a move
        if initial_index + 1 < path.path.len() {
            let component = PathComponent { slice: &path.path[initial_index..path.path.len()] };
            path_components.push(component);
        }

        path_components
    }
}

impl<'a> Geo for PathComponent<'a> {
    type Point = Coord2;
}
