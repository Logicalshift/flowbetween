use flo_canvas::*;

pub const SPRITE_BEZIER_POINT: SpriteId         = SpriteId(1);
pub const SPRITE_BEZIER_CONTROL_POINT: SpriteId = SpriteId(2);
pub const SPRITE_SELECTION_OUTLINE: SpriteId    = SpriteId(3);

/// Sprites with IDs higher than this are not allocated to tools
pub const SPRITE_FIRST_UNALLOCATED: SpriteId    = SpriteId(1024);
