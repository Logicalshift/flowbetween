use flo_draw::canvas::*;

#[inline] pub fn color_tool_border() -> Color           { Color::Rgba(0.6, 0.6, 0.6, 1.0) }
#[inline] pub fn color_tool_shadow() -> Color           { Color::Rgba(0.4, 0.4, 0.4, 0.4) }
#[inline] pub fn color_tool_outline() -> Color          { Color::Rgba(0.4, 0.7, 1.0, 1.0) }
#[inline] pub fn color_tool_background() -> Color       { Color::Rgba(0.7, 0.7, 0.7, 0.9) }

#[inline] pub fn color_tool_dock_background() -> Color  { Color::Rgba(0.2, 0.2, 0.2, 0.8) }
#[inline] pub fn color_tool_dock_outline() -> Color     { Color::Rgba(0.8, 0.8, 0.8, 1.0) }
#[inline] pub fn color_tool_dock_highlight() -> Color   { Color::Rgba(0.3, 0.3, 0.3, 1.0) }
#[inline] pub fn color_tool_dock_selected() -> Color    { Color::Rgba(0.0, 0.0, 0.0, 1.0) }
