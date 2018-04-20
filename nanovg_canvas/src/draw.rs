use super::path::*;
use super::paint::*;
use super::viewport::*;

use flo_canvas;
use flo_canvas::Draw;
use nanovg::*;
use nanovg;

///
/// Represents state associated with sending canvas drawing commands to a nanovg frame
/// 
pub struct NanoVgDrawingState {
    /// The size of the framebuffer in pixels
    viewport: NanoVgViewport,

    /// Pending path instructions
    path: Vec<NanoVgPath>,

    /// Current stroke paint option
    stroke: NanoVgPaint,

    /// Current fill paint option
    fill: NanoVgPaint,

    /// Current fill options
    fill_options: FillOptions,

    /// Current stroke options
    stroke_options: StrokeOptions,

    /// Current path options
    path_options: PathOptions
}

impl NanoVgDrawingState {
    ///
    /// Creates a new NanoVgDrawing state
    /// 
    pub fn new(viewport: NanoVgViewport) -> NanoVgDrawingState {
        NanoVgDrawingState {
            viewport:           viewport,
            path:               vec![],
            stroke:             NanoVgPaint::Color(nanovg::Color::new(0.0, 0.0, 0.0, 1.0)),
            fill:               NanoVgPaint::Color(nanovg::Color::new(0.0, 0.0, 0.0, 1.0)),
            fill_options:       FillOptions { antialias: true },
            stroke_options:     StrokeOptions { width: 1.0, line_cap: LineCap::Butt, line_join: LineJoin::Miter, miter_limit: 16.0, antialias: true },
            path_options:       PathOptions { clip: Clip::None, composite_operation: CompositeOperation::Basic(BasicCompositeOperation::SourceOver), alpha: 1.0, transform: Some(viewport.to_transform()) }
        }
    }

    ///
    /// If there are any uncommitted drawing actions from a previous draw() call, ensures that they are committed to the specified frame
    /// 
    /// (Useful when changing layers, for example)
    /// 
    pub fn commit<'a>(&mut self, frame: &Frame<'a>) {

    }

    ///
    /// Renders the current path to a NanoVG path
    /// 
    fn render_path(&self, path: &Path) {
        self.path.iter().for_each(|item| item.add_to_path(path));
    }

    ///
    /// Fills a path on the current frame
    /// 
    fn fill_path<'a>(&self, frame: &Frame<'a>) {
        frame.path(|path| {
            self.render_path(&path);
            path.fill(&self.fill, FillOptions { antialias: true });
        },
        self.path_options.clone());
    }

    ///
    /// Draws an outline for a path on the current frame
    /// 
    fn stroke_path<'a>(&self, frame: &Frame<'a>) {
        frame.path(|path| {
            let opt = &self.stroke_options;

            self.render_path(&path);
            path.stroke(&self.fill, StrokeOptions { width: opt.width, line_cap: opt.line_cap, line_join: opt.line_join, miter_limit: opt.miter_limit, antialias: opt.antialias });
        },
        self.path_options.clone());
    }

    ///
    /// Converts a canvas blending mode into a nanovg blending mdoe
    /// 
    fn blend_mode(canvas_mode: flo_canvas::BlendMode) -> CompositeOperation {
        use flo_canvas::BlendMode::*;

        match canvas_mode {
            SourceOver          => CompositeOperation::Basic(BasicCompositeOperation::SourceOver),
            SourceIn            => CompositeOperation::Basic(BasicCompositeOperation::SourceIn),
            SourceOut           => CompositeOperation::Basic(BasicCompositeOperation::SourceOut),
            DestinationOver     => CompositeOperation::Basic(BasicCompositeOperation::DestinationOver),
            DestinationIn       => CompositeOperation::Basic(BasicCompositeOperation::DestinationIn),
            DestinationOut      => CompositeOperation::Basic(BasicCompositeOperation::DestinationOut),
            SourceAtop          => CompositeOperation::Basic(BasicCompositeOperation::Atop),
            DestinationAtop     => CompositeOperation::Basic(BasicCompositeOperation::DestinationAtop),

            Multiply            |   // TODO: I think these are all probably possible with other composite operations but are less eimple than the ones above
            Screen              |
            Darken              |
            Lighten             => CompositeOperation::Basic(BasicCompositeOperation::SourceOver)
        }
    }

    ///
    /// Computes the transformation to apply for a particular canvas height
    /// 
    fn height_transform(height: f32) -> Transform {
        let mut ratio_x = 2.0/height;
        let ratio_y     = ratio_x;

        if height < 0.0 {
            ratio_x = -ratio_x;
        }

        let mut result  = Transform::new();
        result.scale(ratio_x, ratio_y);

        result
    }

    ///
    /// Computes a transform to make a particular region centered in the viewport
    /// 
    fn center_transform(current_matrix: &Transform, viewport: &NanoVgViewport, minx: f32, miny: f32, maxx: f32, maxy: f32) -> Transform {
        let pixel_width     = viewport.width as f32;
        let pixel_height    = viewport.height as f32;

        let xx = current_matrix.matrix[0];
        let yx = current_matrix.matrix[1];
        let xy = current_matrix.matrix[2];
        let yy = current_matrix.matrix[3];
        let x0 = current_matrix.matrix[4];
        let y0 = current_matrix.matrix[5];

        // Get the current scaling of this canvas
        let mut xscale = (xx*xx + yx*yx).sqrt();
        let mut yscale = (xy*xy + yy*yy).sqrt();
        if xscale == 0.0 { xscale = 1.0; }
        if yscale == 0.0 { yscale = 1.0; }

        // Current X, Y coordinates (centered)
        let cur_x = (x0-(pixel_width/2.0))/xscale;
        let cur_y = (y0-(pixel_height/2.0))/yscale;
        
        // New center coordinates
        let center_x = (minx+maxx)/2.0;
        let center_y = (miny+maxy)/2.0;

        // Compute the offsets and transform the canvas
        let x_offset = cur_x - center_x;
        let y_offset = cur_y - center_y;

        let x_offset = x_offset + (viewport.viewport_x as f32)/xscale;
        let y_offset = y_offset + (viewport.viewport_y as f32)/xscale;

        // Generate the result matrix
        let mut result = Transform::new();
        result.translate(x_offset, y_offset)
    }

    ///
    /// Performs the canvas height operation
    /// 
    fn canvas_height(&mut self, height: f32) {
        let height = Self::height_transform(height);
        self.path_options.transform = self.path_options.transform.clone()
            .map(|transform| height * transform);
    }

    ///
    /// Performs the center region operation
    /// 
    fn center_region(&mut self, minx: f32, miny: f32, maxx: f32, maxy: f32) {
        let transform = self.path_options.transform.take().unwrap_or(self.viewport.to_transform());

        let center = Self::center_transform(&transform, &self.viewport, minx, miny, maxx, maxy);;
        self.path_options.transform = self.path_options.transform.clone()
            .map(|transform| center * transform);
    }

    ///
    /// Performs a drawing action on the specified frame
    /// 
    pub fn draw<'a>(&mut self, drawing: Draw, frame: &Frame<'a>) {
        use self::Draw::*;

        match drawing {
            NewPath                                     => { self.path = vec![] },
            Move(x, y)                                  => { self.path.push(NanoVgPath::MoveTo(x, y)); },
            Line(x, y)                                  => { self.path.push(NanoVgPath::LineTo(x, y)); },
            BezierCurve(pos, cp1, cp2)                  => { self.path.push(NanoVgPath::CubicBezier(pos, cp1, cp2)); },
            ClosePath                                   => { self.path.push(NanoVgPath::Close); },
            Fill                                        => { self.fill_path(frame); },
            Stroke                                      => { self.stroke_path(frame); },
            LineWidth(width)                            => { self.stroke_options.width = width; },
            LineWidthPixels(width)                      => { },
            LineJoin(join)                              => { },
            LineCap(cap)                                => { },
            NewDashPattern                              => { },
            DashLength(len)                             => { /* Dashed paths are not supported by nanovg at the moment */ },
            DashOffset(offset)                          => { /* Dashed paths are not supported by nanovg */ },
            FillColor(col)                              => { self.fill = col.into(); },
            StrokeColor(col)                            => { self.stroke = col.into(); },
            BlendMode(blend)                            => { self.path_options.composite_operation = Self::blend_mode(blend); },
            IdentityTransform                           => { self.path_options.transform = Some(self.viewport.to_transform()) },
            CanvasHeight(height)                        => { self.canvas_height(height); },
            CenterRegion((minx, miny), (maxx, maxy))    => { self.center_region(minx, miny, maxx, maxy); },
            MultiplyTransform(transform)                => { },
            Unclip                                      => { },
            Clip                                        => { /* TODO: store the current path as the clipping path, write to a clip buffer, then use that with an image source to draw the final result */ },
            Store                                       => { },
            Restore                                     => { },
            FreeStoredBuffer                            => { },
            PushState                                   => { },
            PopState                                    => { },
            ClearCanvas                                 => { },
            Layer(layer_id)                             => { },
            LayerBlend(layer_id, mode)                  => { },
            ClearLayer                                  => { }
        }
    }
}
