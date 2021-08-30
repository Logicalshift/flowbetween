use crate::animation_path::*;

use flo_canvas::*;

///
/// Converts drawing on a single layer to paths
///
pub struct LayerDrawingToPaths {

}

impl LayerDrawingToPaths {
    ///
    /// Creates a new drawing-to-paths converter
    ///
    pub fn new() -> LayerDrawingToPaths {
        LayerDrawingToPaths {
        }
    }

    ///
    /// Sends some drawing instructions to this layer
    ///
    pub fn draw<'a, DrawIter: IntoIterator<Item=&'a Draw>>(&mut self, drawing: DrawIter) -> impl Iterator<Item=AnimationPath> {
        for draw in drawing {
            use Draw::*;

            match draw {
                Path(path_op)                                   => { unimplemented!(); },
                Fill                                            => { unimplemented!(); },
                Stroke                                          => { unimplemented!(); },

                LineWidth(width)                                => { unimplemented!(); },
                LineWidthPixels(pixel_width)                    => { unimplemented!(); },
                LineJoin(line_join)                             => { unimplemented!(); },
                LineCap(line_cap)                               => { unimplemented!(); },
                NewDashPattern                                  => { unimplemented!(); },
                DashLength(dash_length)                         => { unimplemented!(); },
                DashOffset(dash_offset)                         => { unimplemented!(); },
                StrokeColor(stroke_color)                       => { unimplemented!(); },

                FillColor(fill_color)                           => { unimplemented!(); },
                FillTexture(fill_texture, (x1, y1), (x2, y2))   => { unimplemented!(); },
                FillGradient(gradient_id, (x1, y1), (x2, y2))   => { unimplemented!(); },
                FillTransform(fill_transform)                   => { unimplemented!(); },
                WindingRule(winding_rule)                       => { unimplemented!(); },

                MultiplyTransform(transform)                    => { unimplemented!(); },

                PushState                                       => { unimplemented!(); },
                PopState                                        => { unimplemented!(); },

                ClearCanvas(canvas_colour)                      => { unimplemented!(); },
                ClearLayer                                      => { unimplemented!(); },
                ClearAllLayers                                  => { unimplemented!(); },

                BlendMode(blend_mode)                           => { debug_assert!(false, "Blend modes not yet supported in an animation layer"); },

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
        }

        vec![].into_iter()
    }
}
