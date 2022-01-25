#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct OverlayLayerId(pub u64);

pub const OVERLAY_ELEMENTS: OverlayLayerId      = OverlayLayerId(0);
pub const OVERLAY_TOOL: OverlayLayerId          = OverlayLayerId(1);
pub const OVERLAY_ONIONSKINS: OverlayLayerId    = OverlayLayerId(2);
