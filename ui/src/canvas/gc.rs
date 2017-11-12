use super::*;
use std::mem;

///
/// A GC ("graphics context") is a short-hand way to write a bunch of operations to a canvas all at once
/// 
pub struct Gc<'a> {
    /// The canvas that this GC will write to
    canvas: &'a mut Canvas,

    /// The commands that are pending for the canvas
    pending: Vec<Draw>
}

use self::BlendMode;
use self::LineJoin;
use self::LineCap;
use self::Draw::*;

impl<'a> Gc<'a> {
    ///
    /// Creates a new graphics context from a canvas
    ///
    pub fn new(canvas: &'a mut Canvas) -> Gc<'a> {
        Gc {
            canvas:     canvas,
            pending:    vec![]
        }
    }

    pub fn new_path(&mut self)                  { self.pending.push(NewPath) }
    pub fn move_to(&mut self, x: f32, y: f32)   { self.pending.push(Move(x, y)) }
    pub fn line_to(&mut self, x: f32, y: f32)   { self.pending.push(Line(x, y)) }

    pub fn bezier_curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) {
        self.pending.push(BezierCurve((x1, y1), (x2, y2), (x3, y3)));
    }

    pub fn rect(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.pending.push(Rect((x1, y1), (x2, y2)));
    }

    pub fn fill(&mut self)                          { self.pending.push(Fill) }
    pub fn stroke(&mut self)                        { self.pending.push(Stroke) }
    pub fn line_width(&mut self, width: f32)        { self.pending.push(LineWidth(width)) }
    pub fn line_join(&mut self, join: LineJoin)     { self.pending.push(LineJoin(join)) }
    pub fn line_cap(&mut self, cap: LineCap)        { self.pending.push(LineCap(cap)) }
    pub fn dash_length(&mut self, length: f32)      { self.pending.push(DashLength(length)) }
    pub fn dash_offset(&mut self, offset: f32)      { self.pending.push(DashOffset(offset)) }
    pub fn fill_color(&mut self, col: Color)        { self.pending.push(FillColor(col)) }
    pub fn stroke_color(&mut self, col: Color)      { self.pending.push(StrokeColor(col)) }
    pub fn blend_mode(&mut self, mode: BlendMode)   { self.pending.push(BlendMode(mode)) }
    pub fn identity_transform(&mut self)            { self.pending.push(IdentityTransform) }
    pub fn canvas_height(&mut self, height: f32)    { self.pending.push(CanvasHeight(height)) }
    pub fn transform(&mut self, transform: Transform2D) { self.pending.push(MultiplyTransform(transform)) }
    pub fn unclip(&mut self)                        { self.pending.push(Unclip) }
    pub fn clip(&mut self)                          { self.pending.push(Clip) }
    pub fn store(&mut self)                         { self.pending.push(Store) }
    pub fn restore(&mut self)                       { self.pending.push(Restore) }
    pub fn push_state(&mut self)                    { self.pending.push(PushState) }
    pub fn pop_state(&mut self)                     { self.pending.push(PopState) }

    pub fn clear_canvas(&mut self) {
        // When clearing the canvas, all the other commands cease to matter
        self.pending.clear();
        self.pending.push(ClearCanvas);
    }
}

impl<'a> Drop for Gc<'a> {
    fn drop(&mut self) {
        // Draw any pending commands
        let mut pending = vec![];
        mem::swap(&mut self.pending, &mut pending);

        self.canvas.draw(pending)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use futures::executor;

    #[test]
    fn can_draw_line_with_gc() {
        let mut canvas = Canvas::new();

        {
            let mut gc = Gc::new(&mut canvas);

            gc.blend_mode(BlendMode::SourceOver);
            gc.new_path();
            gc.move_to(0.0,0.0);
            gc.line_to(100.0, 100.0);
            gc.stroke();
        }

        let mut stream  = executor::spawn(canvas.stream());

        assert!(stream.wait_stream() == Some(Ok(Draw::ClearCanvas)));
        assert!(stream.wait_stream() == Some(Ok(Draw::BlendMode(BlendMode::SourceOver))));
        assert!(stream.wait_stream() == Some(Ok(Draw::NewPath)));
        assert!(stream.wait_stream() == Some(Ok(Draw::Move(0.0,0.0))));
        assert!(stream.wait_stream() == Some(Ok(Draw::Line(100.0, 100.0))));
        assert!(stream.wait_stream() == Some(Ok(Draw::Stroke)));
    }
}