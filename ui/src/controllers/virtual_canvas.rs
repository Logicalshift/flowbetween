use super::super::control::*;
use super::super::binding_canvas::*;
use super::super::resource_manager::*;

use canvas::*;
use binding::*;

use std::sync::*;

///
/// Provides a way to define a virtual canvas (produces a control that can be embedded
/// in another control that uses the `VirtualScroll` action)
///
pub struct VirtualCanvas<DrawRegion: Fn(&mut GraphicsPrimitives, (f32, f32)) -> ()> {
    /// The sub-canvases for this control
    canvas_resources: Arc<ResourceManager<BindingCanvas>>,

    /// The canvases that are currently being displayed in this virtual canvas
    canvases: Binding<Vec<Vec<Resource<BindingCanvas>>>>,

    /// The top-left grid coordinate
    top_left: Binding<(u32, u32)>,

    /// The width and height of the grid
    grid_size: Binding<(u32, u32)>,

    /// The size of a tile canvas
    tile_size: Binding<(f32, f32)>,

    /// Draws a section of the virtual canvas
    draw_region: DrawRegion,

    /// Binding for the control
    control: Arc<Bound<Control>>
}

impl<DrawRegion: Fn(&mut GraphicsPrimitives, (f32, f32)) -> ()> VirtualCanvas<DrawRegion> {
    ///
    /// Creates a new virtual canvas
    /// 
    pub fn new(canvas_resources: Arc<ResourceManager<BindingCanvas>>, draw_region: DrawRegion) -> VirtualCanvas<DrawRegion> {
        let canvases    = bind(vec![]);
        let top_left    = bind((0, 0));
        let grid_size   = bind((0, 0));
        let tile_size   = bind((256.0, 256.0));
        let control     = Self::make_control(&canvases, &top_left, &grid_size, &tile_size);

        VirtualCanvas {
            canvas_resources:   canvas_resources,
            canvases:           canvases,
            top_left:           top_left,
            grid_size:          grid_size,
            tile_size:          tile_size,
            draw_region:        draw_region,
            control:            control
        }
    }

    ///
    /// Retrieves the control that renders this virtual canvas
    /// 
    pub fn control(&self) -> Arc<Bound<Control>> {
        Arc::clone(&self.control)
    }

    ///
    /// Handles a virtual scroll event
    /// 
    pub fn virtual_scroll(&self, tile_size: (f32, f32), top_left: (u32, u32), grid_size: (u32, u32)) {
        if self.tile_size.get() != tile_size {
            // Tile size mainly affects how the regions are drawn
            self.tile_size.clone().set(tile_size);
        }
   
        if self.top_left.get() != top_left {
            // Top-left coordinate affects what is drawn in the various canvases
            // We need to re-order the canvases to avoid having to redraw all of the tiles
            self.top_left.clone().set(top_left);
        }

        if self.grid_size.get() != grid_size {
            // Grid size affects the number of canvases we're drawing overall
            self.grid_size.clone().set(grid_size);
        }
    }

    ///
    /// Creates the control binding for this virtual canvas
    /// 
    fn make_control(canvases: &Binding<Vec<Vec<Resource<BindingCanvas>>>>, top_left: &Binding<(u32, u32)>, grid_size: &Binding<(u32, u32)>, tile_size: &Binding<(f32, f32)>) -> Arc<Bound<Control>> {
        // Clone the bindings
        let canvases    = Binding::clone(canvases);
        let top_left    = Binding::clone(top_left);
        let grid_size   = Binding::clone(grid_size);
        let tile_size   = Binding::clone(tile_size);

        // Bind a new control
        Arc::new(computed(move || {
            Control::empty()
        }))
    }
}
