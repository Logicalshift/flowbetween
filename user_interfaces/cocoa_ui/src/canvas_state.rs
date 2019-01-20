use super::core_graphics_ffi::*;

use flo_canvas::*;

///
/// The values stores in a canvas state
///
#[derive(Clone)]
struct CanvasStateValues {
    color_space:    CFRef<CGColorSpaceRef>,
    fill_color:     CFRef<CGColorRef>,
    stroke_color:   CFRef<CGColorRef>,
    transform:      CGAffineTransform
}

///
/// Represents current state of a canvas
///
#[derive(Clone)]
pub struct CanvasState {
    context:        Option<CFRef<CGContextRef>>,
    values:         CanvasStateValues,
    stack:          Vec<CanvasStateValues>
}

impl CanvasState {
    ///
    /// Creates a new canvas state
    ///
    pub fn new(color_space: CFRef<CGColorSpaceRef>) -> CanvasState {
        unsafe {
            let fill_color      = CFRef::from(CGColorCreate(*color_space, [0.0, 0.0, 0.0, 0.0].as_ptr()));
            let stroke_color    = fill_color.clone();
            let transform       = CGAffineTransformIdentity;

            CanvasState {
                context:    None,
                values:     CanvasStateValues {
                    color_space:    color_space,
                    fill_color:     fill_color,
                    stroke_color:   stroke_color,
                    transform:      transform
                },
                stack:      vec![]
            }
        }
    }

    ///
    /// Activates a new graphics context with this state
    ///
    pub fn activate_context(&mut self, new_context: CFRef<CGContextRef>) {
        unsafe {
            // Deactivate any existing context
            self.deactivate_context();

            // Save the GState of the new context: the current transformation becomes the base transformation
            // (We don't use the GState for anything else. We need to be able to set the transformation to new
            // values but Cocoa only has the ability to multiply in new transformations. Finding the
            // transformation that updates the current one is possible but prone to accumulating errors.
            // This makes updating the transformation fairly slow due to the need to restore the state and 
            // also makes the state very awkward to use for any other purpose)
            CGContextSaveGState(*new_context);

            // Store the new context
            self.context = Some(new_context);

            // Make sure the current state is applied to it
            self.reapply_state();
        }
    }

    ///
    /// Deactivates the current graphics context
    ///
    pub fn deactivate_context(&mut self) {
        if let Some(ref context) = self.context {
            unsafe {
                // Restore the GState
                CGContextRestoreGState(**context);
            }
        }

        self.context = None;
    }

    ///
    /// Returns the current affine transform for this state
    ///
    pub fn current_transform(&self) -> CGAffineTransform {
        self.values.transform
    }

    ///
    /// Re-applies the state contained within this object to the current graphics context
    ///
    pub fn reapply_state(&self) {
        unsafe {
            if let Some(ref context) = self.context {
                // Reset the GState and re-save it
                CGContextRestoreGState(**context);
                CGContextSaveGState(**context);

                // Set the values from the current state
                CGContextSetFillColorWithColor(**context, *self.values.fill_color);
                CGContextSetStrokeColorWithColor(**context, *self.values.stroke_color);
                CGContextConcatCTM(**context, self.values.transform);
            }
        }
    }

    ///
    /// Sets the fill color of this state
    ///
    pub fn set_fill_color(&mut self, new_fill_color: &Color) {
        unsafe {
            // Create the fill colour
            let (r, g, b, a)        = new_fill_color.to_rgba_components();
            let new_color           = CFRef::from(CGColorCreate(*self.values.color_space, [r as CGFloat, g as CGFloat, b as CGFloat, a as CGFloat].as_ptr()));

            // Store it in this object
            self.values.fill_color  = new_color;

            // Set in the context
            if let Some(ref context) = self.context {
                CGContextSetFillColorWithColor(**context, *self.values.fill_color);
            }
        }
    }

    ///
    /// Sets the stroke color of this state
    ///
    pub fn set_stroke_color(&mut self, new_stroke_color: &Color) {
        unsafe {
            // Create the fill colour
            let (r, g, b, a)            = new_stroke_color.to_rgba_components();
            let new_color               = CFRef::from(CGColorCreate(*self.values.color_space, [r as CGFloat, g as CGFloat, b as CGFloat, a as CGFloat].as_ptr()));

            // Store it in this object
            self.values.stroke_color    = new_color;

            // Set in the context
            if let Some(ref context) = self.context {
                CGContextSetStrokeColorWithColor(**context, *self.values.stroke_color);
            }
        }
    }

    ///
    /// Sets the transformation matrix for this state
    ///
    pub fn set_transform(&mut self, new_transform: CGAffineTransform) {
        // Cocoa doesn't support setting the transformation matrix directly: we restore the original and reset all the properties
        self.values.transform = new_transform;
        self.reapply_state();
    }

    ///
    /// Saves the current state on the stack
    ///
    pub fn push_state(&mut self) {
        self.stack.push(self.values.clone());
    }

    ///
    /// Restores the current state from the stack
    ///
    pub fn pop_state(&mut self) {
        if let Some(new_values) = self.stack.pop() {
            self.values = new_values;
            self.reapply_state();
        }
    }
}
