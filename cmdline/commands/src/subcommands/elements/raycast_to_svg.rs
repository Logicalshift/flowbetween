use crate::state::*;
use crate::error::*;
use crate::output::*;

use flo_stream::*;
use flo_animation::*;

use futures::prelude::*;

///
/// Outputs a series of SVG files that show raycasting on the specified element
///
pub fn raycast_to_svg<'a>(output: &'a mut Publisher<FloCommandOutput>, state: &'a mut CommandState, element_id: ElementId) -> impl 'a+Future<Output=Result<(), CommandError>>+Send {
    async move {
        // We need to have the frame with the element selected
        let frame       = state.frame();
        let frame       = match frame {
            Some(frame) => frame,
            None        => { return Err(CommandError::NoFrameSelected) }
        };

        // Fetch the element
        let element     = frame.element_with_id(element_id);
        let element     = match element {
            Some(element)   => element,
            None            => { return Err(CommandError::ElementNotFound(element_id)) }
        };

        // Fetch the paths for this element

        Ok(())
    }
}
