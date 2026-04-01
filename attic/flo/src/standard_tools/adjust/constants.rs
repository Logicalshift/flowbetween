use flo_canvas::*;

/// Layer where the current selection is drawn
pub (super) const LAYER_SELECTION: LayerId  = LayerId(0);

/// Layer where the control points and the preview of the region the user is dragging is drawn
pub (super) const LAYER_PREVIEW: LayerId    = LayerId(1);

/// Proximity the pointer should be to a control point to interact with it
pub (super) const MIN_DISTANCE: f64         = 4.0;
