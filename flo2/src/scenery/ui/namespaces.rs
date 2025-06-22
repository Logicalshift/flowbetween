use flo_draw::canvas::*;
use once_cell::sync::{Lazy};

pub static DIALOG_LAYER: Lazy<NamespaceId>  = Lazy::new(|| NamespaceId::new());
pub static PHYSICS_LAYER: Lazy<NamespaceId> = Lazy::new(|| NamespaceId::new());
