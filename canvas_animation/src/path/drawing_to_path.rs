use crate::path::layer_state::*;
use crate::path::animation_path::*;

use flo_canvas::*;

use std::mem;
use std::sync::*;

///
/// Converts drawing on a single layer to paths
///
pub struct LayerDrawingToPaths {
    state:          LayerState,
    state_stack:    Vec<LayerState>
}

impl LayerDrawingToPaths {
    ///
    /// Creates a new drawing-to-paths converter
    ///
    pub fn new() -> LayerDrawingToPaths {
        LayerDrawingToPaths {
            state:          LayerState::default(),
            state_stack:    vec![]
        }
    }

    ///
    /// Sends some drawing instructions to this layer
    ///
    pub fn draw<'a, DrawIter: 'a+IntoIterator<Item=&'a Draw>>(&'a mut self, drawing: DrawIter) -> impl 'a+Iterator<Item=AnimationPath> {
        drawing.into_iter()
            .flat_map(move |draw| {
                use Draw::*;

                match draw {
                    Path(PathOp::NewPath)                           => { self.state.current_path.clear(); self.state.cached_path = None; }
                    Path(path_op)                                   => { 
                        if let Some(path) = self.state.cached_path.take() { 
                            self.state.current_path = (*path).clone(); 
                        } 
                        self.state.current_path.push(path_op.clone()); 
                    },

                    Fill                                            => {
                        // Retrieve the current path (re-use if already cached)
                        let path = if let Some(path) = &self.state.cached_path {
                            Arc::clone(path)
                        } else {
                            let path                = Arc::new(mem::take(&mut self.state.current_path));
                            self.state.cached_path  = Some(Arc::clone(&path));
                            path
                        };

                        // Turn the fill state into attributes
                        let attributes = (&self.state.fill).into();

                        return Some(AnimationPath {
                            appearance_time:    self.state.current_time,
                            disappearance_time: None,
                            attributes:         attributes,
                            path:               path
                        });
                    },
                    Stroke                                          => {
                        // Retrieve the current path (re-use if already cached)
                        let path = if let Some(path) = &self.state.cached_path {
                            Arc::clone(path)
                        } else {
                            let path                = Arc::new(mem::take(&mut self.state.current_path));
                            self.state.cached_path  = Some(Arc::clone(&path));
                            path
                        };

                        // Turn the stroke state into attributes
                        let attributes = (&self.state.stroke).into();

                        return Some(AnimationPath {
                            appearance_time:    self.state.current_time,
                            disappearance_time: None,
                            attributes:         attributes,
                            path:               path
                        });
                    },

                    StrokeColor(stroke_color)                       => { self.state.stroke.color        = *stroke_color; },
                    LineWidth(width)                                => { self.state.stroke.width        = StrokeWidth::CanvasCoords(*width); },
                    LineWidthPixels(pixel_width)                    => { self.state.stroke.width        = StrokeWidth::PixelCoords(*pixel_width); },
                    LineJoin(line_join)                             => { self.state.stroke.line_join    = *line_join; },
                    LineCap(line_cap)                               => { self.state.stroke.line_cap     = *line_cap; },
                    NewDashPattern                                  => { self.state.stroke.dash_pattern = None; },
                    DashLength(dash_length)                         => { self.state.stroke.dash_pattern.get_or_insert_with(|| (0.0, vec![])).1.push(*dash_length); },
                    DashOffset(dash_offset)                         => { self.state.stroke.dash_pattern.get_or_insert_with(|| (0.0, vec![])).0 = *dash_offset; },

                    FillColor(fill_color)                           => { self.state.fill.color          = FillStyle::Solid(*fill_color); self.state.fill.transform = None; },
                    FillTexture(fill_texture, (x1, y1), (x2, y2))   => { self.state.fill.color          = FillStyle::Texture(*fill_texture, (*x1, *y1), (*x2, *y2)); self.state.fill.transform = None; },
                    FillGradient(gradient_id, (x1, y1), (x2, y2))   => { self.state.fill.color          = FillStyle::Gradient(*gradient_id, (*x1, *y1), (*x2, *y2)); self.state.fill.transform = None; },
                    WindingRule(winding_rule)                       => { self.state.fill.winding_rule   = *winding_rule; },
                    FillTransform(fill_transform)                   => {
                        let transform = self.state.fill.transform.unwrap_or_else(|| Transform2D::identity());
                        let transform = &transform * fill_transform;
                        self.state.fill.transform = Some(transform);
                    },

                    MultiplyTransform(multiply_transform)           => {
                        let transform = self.state.transform.unwrap_or_else(|| Transform2D::identity());
                        let transform = &transform * multiply_transform;
                        self.state.transform = Some(transform);
                    },

                    PushState                                       => { self.state_stack.push(self.state.clone()); },
                    PopState                                        => { if let Some(new_state) = self.state_stack.pop() { self.state = new_state; } },

                    ClearCanvas(_canvas_color)                      => { self.state = LayerState::default(); self.state_stack = vec![]; },
                    ClearLayer                                      => { },
                    ClearAllLayers                                  => { },

                    // flo_draw needs to be updated to support this
                    BlendMode(blend_mode)                           => { debug_assert!(false, "Blend modes not yet supported in an animation layer"); },

                    // Clipping paths need to be preserved across multiple paths in the layer for performance so they require some more thought
                    Unclip                                          |
                    Clip                                            => { debug_assert!(false, "Clipping not yet supported in an animation layer"); },

                    IdentityTransform                               => { debug_assert!(false, "Operation unsupported in an animation layer"); },
                    CanvasHeight(_height)                           => { debug_assert!(false, "Operation unsupported in an animation layer"); },
                    CenterRegion((_x1, _y1), (_x2, _y2))            => { debug_assert!(false, "Operation unsupported in an animation layer"); },

                    Layer(_layer_id)                                => { debug_assert!(false, "Animation layers should not have layer change operations sent to them"); },
                    LayerBlend(_layer_id, _layer_blend)             => { debug_assert!(false, "Animation layers should not have layer change operations sent to them"); },

                    Store                                           => { debug_assert!(false, "Operation unsupported in an animation layer"); },
                    Restore                                         => { debug_assert!(false, "Operation unsupported in an animation layer"); },
                    FreeStoredBuffer                                => { debug_assert!(false, "Operation unsupported in an animation layer"); },

                    SwapLayers(_layer_id1, _layer_id2)              => { debug_assert!(false, "Operation unsupported in an animation layer"); },
                    Sprite(_sprite_id)                              => { debug_assert!(false, "Cannot define sprites on animation layers"); },
                    ClearSprite                                     => { debug_assert!(false, "Operation unsupported in an animation layer"); },
                    SpriteTransform(_sprite_transform)              => { debug_assert!(false, "Operation unsupported in an animation layer"); },
                    DrawSprite(_sprite_id)                          => { debug_assert!(false, "Cannot draw sprites on animation layers"); },
                    Texture(_texture_id, _texture_op)               => { debug_assert!(false, "Operation unsupported in an animation layer"); },

                    Font(_font_id, _font_op)                        => { debug_assert!(false, "Operation unsupported in an animation layer"); },
                    BeginLineLayout(_x, _y, _alignment)             => { debug_assert!(false, "Operation unsupported in an animation layer"); },
                    DrawLaidOutText                                 => { debug_assert!(false, "Operation unsupported in an animation layer"); },
                    DrawText(_font_id, _text, _x, _y)               => { debug_assert!(false, "Operation unsupported in an animation layer"); },

                    Gradient(_gradient_id, _gradient_op)            => { debug_assert!(false, "Operation unsupported in an animation layer"); },

                    StartFrame                                      => { debug_assert!(false, "Operation unsupported in an animation layer"); },
                    ShowFrame                                       => { debug_assert!(false, "Operation unsupported in an animation layer"); },
                    ResetFrame                                      => { debug_assert!(false, "Operation unsupported in an animation layer"); },
                }

                None
            })
    }
}
