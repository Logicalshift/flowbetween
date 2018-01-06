use super::super::brush::*;
use super::super::brush_definition::*;
use super::super::brush_properties::*;
use super::super::brush_drawing_style::*;

use canvas::*;

use std::time::Duration;

///
/// Represents a layer that can be painted upon
/// 
pub trait PaintLayer : Send+Sync {
}
