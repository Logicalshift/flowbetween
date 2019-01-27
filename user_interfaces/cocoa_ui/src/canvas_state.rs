use super::core_graphics_ffi::*;

use flo_canvas::*;

///
/// Possible actions stored in the path for this state
///
#[derive(Copy, Clone)]
enum PathAction {
    Close,
    Move(CGFloat, CGFloat),
    Line(CGFloat, CGFloat),
    Curve(CGFloat, CGFloat, CGFloat, CGFloat, CGFloat, CGFloat)
}

///
/// The values stores in a canvas state
///
#[derive(Clone)]
struct CanvasStateValues {
    color_space:    CFRef<CGColorSpaceRef>,
    fill_color:     CFRef<CGColorRef>,
    stroke_color:   CFRef<CGColorRef>,
    transform:      CGAffineTransform,
    blend_mode:     CGBlendMode,
    layer_id:       u32,
    line_width:     CGFloat,
    path:           Vec<PathAction>,
    clip:           Option<Vec<PathAction>>
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
                    transform:      transform,
                    blend_mode:     CGBlendMode::Normal,
                    layer_id:       0,
                    line_width:     1.0,
                    path:           vec![],
                    clip:           None
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
    /// Performs a set of actions with the native 'pixel' transform instead of the one set in this state
    ///
    pub fn with_native_transform<ActionFn: Fn(CGContextRef) -> ()>(&mut self, action: ActionFn) {
        if let Some(ref context) = self.context {
            unsafe {
                // Reset the GState to its default (which will have no transform applied)
                CGContextRestoreGState(**context);
                CGContextSaveGState(**context);

                // Run the actions
                action(**context);

                // Reset the context 
                CGContextRestoreGState(**context);
                CGContextSaveGState(**context);
                self.reapply_state();
            }
        }
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
                CGContextConcatCTM(**context, self.values.transform);
                CGContextSetBlendMode(**context, self.values.blend_mode);
                CGContextSetFillColorWithColor(**context, *self.values.fill_color);
                CGContextSetStrokeColorWithColor(**context, *self.values.stroke_color);
                CGContextSetLineWidth(**context, self.values.line_width);
            }
        }
    }

    ///
    /// Starts a new path
    ///
    #[inline] pub fn begin_path(&mut self) {
        self.values.path = vec![];
    }

    ///
    /// Adds a move action to the current path
    ///
    #[inline] pub fn path_move(&mut self, x: CGFloat, y: CGFloat) {
        self.values.path.push(PathAction::Move(x, y));
    }

    ///
    /// Adds a line action to the current path
    ///
    #[inline] pub fn path_line(&mut self, x: CGFloat, y: CGFloat) {
        self.values.path.push(PathAction::Line(x, y));
    }

    ///
    /// Adds a bezier curve action to the current path
    ///
    #[inline] pub fn path_bezier_curve(&mut self, cp1: (CGFloat, CGFloat), cp2: (CGFloat, CGFloat), end: (CGFloat, CGFloat)) {
        self.values.path.push(PathAction::Curve(cp1.0, cp1.1, cp2.0, cp2.1, end.0, end.1));
    }

    ///
    /// Adds a bezier path 'close' action to the current path
    ///
    #[inline] pub fn path_close(&mut self) {
        self.values.path.push(PathAction::Close);
    }

    ///
    /// Loads the current path into the context
    ///
    pub fn load_path(&self) {
        unsafe {
            if let Some(ref context) = self.context {
                CGContextBeginPath(**context);

                for action in self.values.path.iter() {
                    use self::PathAction::*;

                    match action {
                        Move(x, y)                          => CGContextMoveToPoint(**context, *x, *y),
                        Line(x, y)                          => CGContextAddLineToPoint(**context, *x, *y),
                        Curve(c1x, c1y, c2x, c2y, ex, ey)   => CGContextAddCurveToPoint(**context, *c1x, *c1y, *c2x, *c2y, *ex, *ey),
                        Close                               => CGContextClosePath(**context)
                    }
                }
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
    /// Sets the line width
    ///
    pub fn set_line_width(&mut self, line_width: CGFloat) {
        unsafe {
            self.values.line_width = line_width;

            if let Some(ref context) = self.context {
                CGContextSetLineWidth(**context, line_width);
            }
        }
    }

    ///
    /// Sets the blend mode
    ///
    pub fn set_blend_mode(&mut self, blend_mode: &BlendMode) {
        unsafe {
            // Store the blend mode in the state
            self.values.blend_mode = (*blend_mode).into();

            // Set it in the context
            if let Some(ref context) = self.context {
                CGContextSetBlendMode(**context, self.values.blend_mode);
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
    /// Sets the layer that we should draw to for this context
    ///
    pub fn set_layer_id(&mut self, layer_id: u32) {
        self.values.layer_id = layer_id;
    }

    ///
    /// Retrieves the active layer ID
    ///
    pub fn layer_id(&self) -> u32 {
        self.values.layer_id
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

impl From<BlendMode> for CGBlendMode {
    fn from(blendmode: BlendMode) -> CGBlendMode {
        match blendmode {
            BlendMode::SourceOver       => CGBlendMode::Normal,
            BlendMode::SourceIn         => CGBlendMode::SourceIn,
            BlendMode::SourceOut        => CGBlendMode::SourceOut,
            BlendMode::DestinationOver  => CGBlendMode::DestinationOver,
            BlendMode::DestinationIn    => CGBlendMode::DestinationIn,
            BlendMode::DestinationOut   => CGBlendMode::DestinationOut,
            BlendMode::SourceAtop       => CGBlendMode::SourceAtop,
            BlendMode::DestinationAtop  => CGBlendMode::DestinationAtop,
            BlendMode::Multiply         => CGBlendMode::Multiply,
            BlendMode::Screen           => CGBlendMode::Screen,
            BlendMode::Darken           => CGBlendMode::Darken,
            BlendMode::Lighten          => CGBlendMode::Lighten
        }
    }
}
