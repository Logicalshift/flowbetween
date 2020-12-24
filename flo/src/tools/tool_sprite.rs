use flo_canvas::*;

pub const SPRITE_BEZIER_POINT: SpriteId                     = SpriteId(1);
pub const SPRITE_SELECTED_BEZIER_POINT: SpriteId            = SpriteId(2);
pub const SPRITE_BEZIER_CONTROL_POINT: SpriteId             = SpriteId(3);
pub const SPRITE_SELECTED_BEZIER_CONTROL_POINT: SpriteId    = SpriteId(4);
pub const SPRITE_SELECTION_OUTLINE: SpriteId                = SpriteId(5);

/// Sprites with IDs higher than this are not allocated to tools
pub const SPRITE_FIRST_UNALLOCATED: SpriteId    = SpriteId(1024);
