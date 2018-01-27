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
    draw_region: Arc<DrawRegion>,

    /// Binding for the control
    control: Arc<Bound<Control>>
}

impl<DrawRegion: 'static+Send+Sync+Fn(&mut GraphicsPrimitives, (f32, f32)) -> ()> VirtualCanvas<DrawRegion> {
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
            draw_region:        Arc::new(draw_region),
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
            self.reorder_canvases(self.top_left.get(), top_left);

            self.top_left.clone().set(top_left);
        }

        if self.grid_size.get() != grid_size {
            // Grid size affects the number of canvases we're drawing overall
            self.resize_canvases(grid_size);

            self.grid_size.clone().set(grid_size);
        }
    }

    ///
    /// Makes a canvas at a particular grid position
    /// 
    fn make_canvas(&self, pos: (u32, u32)) -> Resource<BindingCanvas> {
        let draw_region = Arc::clone(&self.draw_region);
        let tile_size   = self.tile_size.clone();

        // Create a new canvas to draw this particular region
        let new_canvas  = BindingCanvas::with_drawing(move |gc| {
            let (width, height) = tile_size.get();
            let (xpos, ypos)    = pos;

            let xpos            = width * (xpos as f32);
            let ypos            = height * (ypos as f32);

            (*draw_region)(gc, (xpos, ypos));
        });

        // Generate a resource. Resource managers keep weak references so we don't need to worry about tidying this up later (unless it's given a name somehow)
        self.canvas_resources.register(new_canvas)
    }

    ///
    /// Updates the canvases grid to match a new grid size
    /// 
    fn resize_canvases(&self, new_grid_size: (u32, u32)) {
        let (left, top)     = self.top_left.get();
        let (width, height) = new_grid_size;
        let mut canvases    = self.canvases.get();

        // Remove any extra rows if we're getting smaller
        canvases.truncate(height as usize);

        // Resize the existing rows
        let mut ypos = top;
        for ref mut row in canvases.iter_mut() {
            // Remove canvases if there are too many
            row.truncate(width as usize);

            // Add canvses if more are needed
            while row.len() < width as usize {
                let xpos = left + row.len() as u32;
                row.push(self.make_canvas((xpos, ypos)));
            }

            // Update ypos to the next row
            ypos += 1;
        }

        // Add new rows
        let mut ypos = top;
        while canvases.len() < height as usize {
            let new_row = (0..width).into_iter()
                .map(|x| left + x)
                .map(|xpos| self.make_canvas((xpos, ypos)))
                .collect();
            canvases.push(new_row);

            ypos += 1;
        }

        // Update the canvases
        self.canvases.clone().set(canvases);
    }

    ///
    /// Takes the canvas array and regenerates it with a new top-left coordinate
    /// 
    fn reorder_canvases(&self, old_top_left: (u32, u32), new_top_left: (u32, u32)) {
        // Fetch the old array and the size of the grid
        let (old_left, old_top) = old_top_left;
        let (new_left, new_top) = new_top_left;
        let (size_x, size_y)    = self.grid_size.get();
        let old_canvases        = self.canvases.get();
        let mut new_canvases    = vec![];

        let (old_left, old_top) = (old_left as i32, old_top as i32);
        let (new_left, new_top) = (new_left as i32, new_top as i32);

        // Generate the new canvas array
        for y in 0..size_y {
            // Start generating this row
            let y               = y as i32;
            let mut this_row    = vec![];

            // Work out the row in the previous set of canvases
            let old_y           = y - (old_top - new_top);

            for x in 0..size_x {
                // Work out the column in the previous set of canvases
                let x       = x as i32;
                let old_x   = x - (old_left - new_left);

                if old_y >= 0 && old_y < old_canvases.len() as i32 {
                    // Row is within the original grid
                    let old_row = &old_canvases[1];

                    if old_x >= 0 && old_x < old_row.len() as i32 {
                        // Use the existing canvas for this grid cell
                        this_row.push(old_row[old_x as usize].clone())
                    } else {
                        // Create a new grid cell
                        this_row.push(self.make_canvas(((x+new_left) as u32, (y+new_top) as u32)));
                    }
                } else {
                    // Create a new grid cell
                    this_row.push(self.make_canvas(((x+new_left) as u32, (y+new_top) as u32)));
                }
            }

            // Add this row to the set
            new_canvases.push(this_row);
        }

        // Update the canvases
        self.canvases.clone().set(new_canvases);
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
