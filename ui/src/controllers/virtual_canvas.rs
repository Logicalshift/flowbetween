use super::super::control::*;
use super::super::binding_canvas::*;
use super::super::resource_manager::*;

use flo_canvas::*;
use flo_binding::*;

use std::sync::*;

///
/// Provides a way to define a virtual canvas (produces a control that can be embedded
/// in another control that uses the `VirtualScroll` action)
///
pub struct VirtualCanvas {
    /// The sub-canvases for this control
    canvas_resources: Arc<ResourceManager<BindingCanvas>>,

    /// The canvases that are currently being displayed in this virtual canvas
    tiles: Binding<Vec<Vec<Resource<BindingCanvas>>>>,

    /// The top-left grid coordinate
    top_left: Binding<(u32, u32)>,

    /// The width and height of the grid
    grid_size: Binding<(u32, u32)>,

    /// The size of a tile canvas
    tile_size: Binding<(f32, f32)>,

    /// Draws a section of the virtual canvas
    draw_region: Arc<dyn Fn(f32, f32) -> Box<dyn Fn(&mut dyn GraphicsPrimitives) -> ()+Send+Sync>+Send+Sync>,

    /// Binding for the control
    control: BindRef<Control>
}

impl VirtualCanvas {
    ///
    /// Creates a new virtual canvas
    ///
    pub fn new<DrawRegion: Fn(f32, f32) -> Box<dyn Fn(&mut dyn GraphicsPrimitives) -> ()+Send+Sync>+Send+Sync+'static>(canvas_resources: Arc<ResourceManager<BindingCanvas>>, draw_region: DrawRegion) -> VirtualCanvas {
        let tiles       = bind(vec![]);
        let top_left    = bind((0, 0));
        let grid_size   = bind((0, 0));
        let tile_size   = bind((256.0, 256.0));
        let control     = Self::make_control(&tiles, &top_left, &tile_size);

        VirtualCanvas {
            canvas_resources:   canvas_resources,
            tiles:              tiles,
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
    pub fn control(&self) -> BindRef<Control> {
        BindRef::clone(&self.control)
    }

    ///
    /// Handles a virtual scroll event
    ///
    /// Callers can use the raw values from the virtual scroll event (the canvases will
    /// always fill the available area), but if they want to hide the canvases loading
    /// in from the user they may want to increase the grid size and move the top-left
    /// corner to allow for a buffer.
    ///
    pub fn virtual_scroll(&self, tile_size: (f32, f32), top_left: (u32, u32), grid_size: (u32, u32)) {
        if self.tile_size.get() != tile_size {
            // Tile size mainly affects how the regions are drawn. We need to remove the existing tiles whenever this changes
            self.tiles.set(vec![]);
            self.tile_size.set(tile_size);
        }

        if self.top_left.get() != top_left {
            // Top-left coordinate affects what is drawn in the various canvases
            // We need to re-order the canvases to avoid having to redraw all of the tiles
            self.reorder_tiles(self.top_left.get(), top_left);

            self.top_left.set(top_left);
        }

        if self.grid_size.get() != grid_size {
            // Grid size affects the number of canvases we're drawing overall
            self.resize_tiles(grid_size);

            self.grid_size.set(grid_size);
        }
    }

    ///
    /// Makes a canvas at a particular grid position
    ///
    fn make_tile(&self, pos: (u32, u32), tile_size: (f32, f32)) -> Resource<BindingCanvas> {
        // Work out the location of this tile
        let (xpos, ypos)    = pos;
        let (width, height) = tile_size;

        let xpos            = width * (xpos as f32);
        let ypos            = height * (ypos as f32);

        // Get the function to draw this region
        let draw_region     = (self.draw_region)(xpos, ypos);

        // Create a new canvas to draw this particular region
        let new_canvas  = BindingCanvas::with_drawing(move |gc| {
            (*draw_region)(gc);
        });

        // Generate a resource. Resource managers keep weak references so we don't need to worry about tidying this up later (unless it's given a name somehow)
        self.canvas_resources.register(new_canvas)
    }

    ///
    /// Updates the canvases grid to match a new grid size
    ///
    fn resize_tiles(&self, new_grid_size: (u32, u32)) {
        let (left, top)     = self.top_left.get();
        let (width, height) = new_grid_size;
        let mut tiles       = self.tiles.get();
        let tile_size       = self.tile_size.get();

        // Remove any extra rows if we're getting smaller
        tiles.truncate(height as usize);

        // Resize the existing rows
        let mut ypos = top;
        for ref mut row in tiles.iter_mut() {
            // Remove canvases if there are too many
            row.truncate(width as usize);

            // Add canvses if more are needed
            while row.len() < width as usize {
                let xpos = left + row.len() as u32;
                row.push(self.make_tile((xpos, ypos), tile_size));
            }

            // Update ypos to the next row
            ypos += 1;
        }

        // Add new rows
        let mut ypos = top;
        while tiles.len() < height as usize {
            let new_row = (0..width).into_iter()
                .map(|x| left + x)
                .map(|xpos| self.make_tile((xpos, ypos), tile_size))
                .collect();
            tiles.push(new_row);

            ypos += 1;
        }

        // Update the canvases
        self.tiles.set(tiles);
    }

    ///
    /// Takes the canvas array and regenerates it with a new top-left coordinate
    ///
    fn reorder_tiles(&self, old_top_left: (u32, u32), new_top_left: (u32, u32)) {
        // Fetch the old array and the size of the grid
        let (old_left, old_top) = old_top_left;
        let (new_left, new_top) = new_top_left;
        let (size_x, size_y)    = self.grid_size.get();
        let old_tiles           = self.tiles.get();
        let tile_size           = self.tile_size.get();
        let mut new_tiles       = vec![];

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

                if old_y >= 0 && old_y < old_tiles.len() as i32 {
                    // Row is within the original grid
                    let old_row = &old_tiles[old_y as usize];

                    if old_x >= 0 && old_x < old_row.len() as i32 {
                        // Use the existing canvas for this grid cell
                        this_row.push(old_row[old_x as usize].clone())
                    } else {
                        // Create a new grid cell
                        this_row.push(self.make_tile(((x+new_left) as u32, (y+new_top) as u32), tile_size));
                    }
                } else {
                    // Create a new grid cell
                    this_row.push(self.make_tile(((x+new_left) as u32, (y+new_top) as u32), tile_size));
                }
            }

            // Add this row to the set
            new_tiles.push(this_row);
        }

        // Update the canvases
        self.tiles.set(new_tiles);
    }

    ///
    /// Creates the control binding for this virtual canvas
    ///
    fn make_control(tiles: &Binding<Vec<Vec<Resource<BindingCanvas>>>>, top_left: &Binding<(u32, u32)>, tile_size: &Binding<(f32, f32)>) -> BindRef<Control> {
        // Clone the bindings
        let tiles       = Binding::clone(tiles);
        let top_left    = Binding::clone(top_left);
        let tile_size   = Binding::clone(tile_size);

        // Bind a new control
        BindRef::new(&computed(move || {
            let (tile_x, tile_y)    = tile_size.get();
            let (left, top)         = top_left.get();
            let tiles               = tiles.get();

            let (left, top)         = (left as f32, top as f32);
            let (left, top)         = (left*tile_x, top*tile_y);

            // Generate the tile controls from the tiles array
            let tile_controls: Vec<_> = tiles.iter()
                .zip((0..tiles.len()).into_iter())
                .map(|(row, ypos)| (row, (ypos as f32) * tile_y + top))
                .map(|(row, ypos)| row.iter()
                    .zip((0..row.len()).into_iter())
                    .map(|(cell, xpos)| (cell, (xpos as f32) * tile_x + left))
                    .map(move |(cell, xpos)| Control::canvas()
                        .with(Bounds {
                            x1: Position::At(xpos),
                            y1: Position::At(ypos),
                            x2: Position::At(xpos+tile_x),
                            y2: Position::At(ypos+tile_y)
                        })
                        .with(cell.clone())
                    )
                )
                .flat_map(|row| row)
                .collect();

            // Turn into a container control with all of the canvases in it
            Control::cropping_container()
                .with(Bounds::fill_all())
                .with(tile_controls)
        }))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn control_is_initially_empty() {
        let resource_manager    = Arc::new(ResourceManager::new());
        let virtual_canvas      = VirtualCanvas::new(resource_manager, |_, _| { Box::new(|_| { }) });

        let control             = virtual_canvas.control();

        assert!(control.get().subcomponents().unwrap().len() == 0);
    }

    #[test]
    fn initial_grid_generates_correctly() {
        let resource_manager    = Arc::new(ResourceManager::new());
        let virtual_canvas      = VirtualCanvas::new(resource_manager, |_, _| { Box::new(|_| { }) });

        let control             = virtual_canvas.control();

        virtual_canvas.virtual_scroll((128.0, 128.0), (0, 0), (6, 2));

        assert!(control.get().subcomponents().unwrap().len() == 12);
    }

    #[test]
    fn grid_scrolls_correctly() {
        let resource_manager    = Arc::new(ResourceManager::new());
        let virtual_canvas      = VirtualCanvas::new(resource_manager, |_, _| { Box::new(|_| { }) });

        let control             = virtual_canvas.control();

        virtual_canvas.virtual_scroll((128.0, 128.0), (0, 0), (6, 2));

        assert!(control.get().subcomponents().unwrap().len() == 12);
        assert!(control.get().subcomponents().unwrap()[0].bounding_box().unwrap().x1 == Position::At(0.0));
        assert!(control.get().subcomponents().unwrap()[0].bounding_box().unwrap().y1 == Position::At(0.0));

        virtual_canvas.virtual_scroll((128.0, 128.0), (2, 4), (6, 2));

        assert!(control.get().subcomponents().unwrap().len() == 12);
        assert!(control.get().subcomponents().unwrap()[0].bounding_box().unwrap().x1 == Position::At(256.0));
        assert!(control.get().subcomponents().unwrap()[0].bounding_box().unwrap().x2 == Position::At(384.0));
        assert!(control.get().subcomponents().unwrap()[0].bounding_box().unwrap().y1 == Position::At(512.0));
        assert!(control.get().subcomponents().unwrap()[0].bounding_box().unwrap().y2 == Position::At(640.0));
    }
}
