use flo_draw as draw;
use flo_draw::canvas as canvas;
use flo_draw::canvas::scenery::*;
use flo_draw::canvas::{GraphicsContext, GraphicsPrimitives};
use egui;
use egui::epaint;

///
/// Converts an egui color to a canvas color
///
#[inline]
pub fn canvas_color(egui_color: &epaint::Color32) -> canvas::Color {
    let rgba = egui::Rgba::from(*egui_color);

    canvas::Color::Rgba(rgba.r(), rgba.g(), rgba.b(), rgba.a())
}

///
/// Writes out the instructions to fill a region
///
pub fn draw_fill(fill: &egui::Color32, drawing: &mut Vec<canvas::Draw>) {
    // Convert to rgba and do nothing if the colour is empty
    let rgba = egui::Rgba::from(*fill);
    if rgba.a() <= 0.0 { return; }

    // Fill with this colour
    drawing.fill_color(canvas::Color::Rgba(rgba.r(), rgba.g(), rgba.b(), rgba.a()));
    drawing.fill();
}

///
/// Writes out the instructions to stroke a region
///
pub fn draw_stroke(stroke: &egui::Stroke, drawing: &mut Vec<canvas::Draw>) {
    // Do nothing if the width is < 0.0
    if stroke.width <= 0.0 { return; }

    // Convert the colour, and do nothing if it's empty
    let rgba = egui::Rgba::from(stroke.color);
    if rgba.a() <= 0.0 { return; }

    // Stroke with this width and colour
    drawing.line_width(stroke.width);
    drawing.stroke_color(canvas::Color::Rgba(rgba.r(), rgba.g(), rgba.b(), rgba.a()));
    drawing.stroke();
}

///
/// Writes out the instructions to stroke a region
///
pub fn draw_path_stroke(stroke: &epaint::PathStroke, drawing: &mut Vec<canvas::Draw>) {
    // Do nothing if the width is < 0.0
    if stroke.width <= 0.0 { return; }

    // TODO: deal with StrokeKind (flo_draw doesn't natively support this)

    // Stroke with this width and colour
    match &stroke.color {
        epaint::ColorMode::Solid(color) => {
            let rgba = egui::Rgba::from(*color);
            if rgba.a() <= 0.0 { return; }

            drawing.line_width(stroke.width);
            drawing.stroke_color(canvas::Color::Rgba(rgba.r(), rgba.g(), rgba.b(), rgba.a()));
            drawing.stroke();
        }

        epaint::ColorMode::UV(callback) => {
            // TODO: flo_draw doesn't have Gouraud shading, so we can't really implement this
        }
    }
}

///
/// Converts an egui texture ID to a canvas texture ID
///
#[inline]
pub fn canvas_texture_id(egui_texture_id: &egui::TextureId) -> canvas::TextureId {
    // Select the flo_draw texture to use (TODO: use separate namespaces for the user and managed textures?)
    match *egui_texture_id {
        epaint::TextureId::Managed(id)  => canvas::TextureId(id as _),
        epaint::TextureId::User(id)     => canvas::TextureId((id | 0x100000000) as _),
    }
}

///
/// Draws a filled region using the specified brush
///
pub fn draw_fill_brush(brush: &epaint::Brush, drawing: &mut Vec<canvas::Draw>, region: ((f32, f32), (f32, f32))) {
    // Select the flo_draw texture to use (TODO: use separate namespaces for the user and managed textures?)
    let texture_id = canvas_texture_id(&brush.fill_texture_id);

    // The UVs go from 0-1 so we need to add/multiply the region as flo_draw works by setting the position of the texture on the canvas
    // TODO: there are monochrome textures that we have to draw using a colour instead
    let mx = region.1.0 - region.0.0;
    let my = region.1.1 - region.0.1;

    let uv_min_x = (brush.uv.min.x + region.0.0) * mx;
    let uv_min_y = (brush.uv.min.y + region.0.1) * my;
    let uv_max_x = (brush.uv.max.x + region.0.0) * mx;
    let uv_max_y = (brush.uv.max.y + region.0.1) * my;

    drawing.fill_texture(texture_id, uv_min_x, uv_min_y, uv_max_x, uv_max_y);
    drawing.fill();
}

///
/// Draws a rectangle shape
///
pub fn draw_rect(rect_shape: &epaint::RectShape, drawing: &mut Vec<canvas::Draw>) {
    // Create the rectangle path
    // TODO: rounded corners
    // TODO: round to pixels if requested
    // TODO: deal with the stroke kind
    drawing.new_path();
    drawing.rect(rect_shape.rect.min.x, rect_shape.rect.min.y, rect_shape.rect.max.x, rect_shape.rect.max.y);

    // TODO: render to a sprite and blur if there's a blur width set
    // Fill with the requested fill colour
    draw_fill(&rect_shape.fill, drawing);
    draw_stroke(&rect_shape.stroke, drawing);

    // Render the texture if there is one
    if let Some(brush) = &rect_shape.brush {
        let bounds = ((rect_shape.rect.min.x, rect_shape.rect.min.y), (rect_shape.rect.max.x, rect_shape.rect.max.y));
        draw_fill_brush(&**brush, drawing, bounds);
    }
}

///
/// Given min/max UV coordinates and canvas coordinates, calculates where the start and end of the texture will appear in canvas coordinates
///
/// (This converts from UV coordinates to how flo_canvas represents texture positions)
///
#[inline]
pub fn texture_pos_for_uv(pos_min: f32, pos_max: f32, uv_min: f32, uv_max: f32) -> (f32, f32) {
    // Solve for uv_min = (pos_min-a)/(b-a), uv_max = (pos_max-a)/(b-a):
    let a = (-pos_min*uv_max + pos_max*uv_min)/(uv_min-uv_max);
    let b = (pos_min*uv_max - pos_max*uv_min - pos_min + pos_max)/(-uv_min+uv_max);

    (a, b)
}

///
/// Creates rendering instructions for text
///
pub fn draw_text(text_shape: &epaint::TextShape, drawing: &mut Vec<canvas::Draw>) {
    // flo_canvas doesn't have an ideal format for the way that egui generates glyphs, so this is probably slower than it could be

    let fallback_color      = canvas_color(&text_shape.fallback_color);
    let texture_id          = canvas::TextureId(0);
    let texture_size        = (2048.0, 64.0);           // TODO: hard coding this for testing, we need to actually store this somewhere, used for converting the UVs
    let mut pos_x           = text_shape.pos.x;
    let mut pos_y           = text_shape.pos.y;
    let mut active_color    = None;

    for row in text_shape.galley.rows.iter() {
        // Draw the glyphs in this row
        for glyph in row.glyphs.iter() {
            // Position of the glyph (coordinates are upside-down)
            let glyph_min_x = pos_x + glyph.pos.x + glyph.uv_rect.offset.x;
            let glyph_min_y = pos_y + glyph.pos.y + glyph.uv_rect.offset.y;
            let glyph_max_x = glyph_min_x + glyph.uv_rect.size.x;
            let glyph_max_y = glyph_min_y + glyph.uv_rect.size.y;

            if glyph_max_x == glyph_min_x || glyph_max_y == glyph_min_y {
                continue;
            }

            // UV coordinates (flo_canvas positions the whole texture, which is more convenient if you're rendering stuff but kind of annoying if you have coords for a GPU so this is a bit involved)

            // Texture coordinate that should appear at glyph_min_x, etc
            let (texture_min_x, texture_max_x) = texture_pos_for_uv(glyph_min_x, glyph_max_x, glyph.uv_rect.min[0] as f32 / 2048.0, glyph.uv_rect.max[0] as f32 / 2048.0);
            let (texture_min_y, texture_max_y) = texture_pos_for_uv(glyph_min_y, glyph_max_y, glyph.uv_rect.min[1] as f32 / 2048.0, glyph.uv_rect.max[1] as f32 / 2048.0);

            // Colour and other formatting is done by looking up the section in the original rendering job
            let section     = glyph.section_index;
            let glyph_color = text_shape.galley.job.sections[section as usize].format.color;
            let glyph_color = if glyph_color == egui::Color32::PLACEHOLDER { fallback_color } else { canvas_color(&glyph_color) };

            // Render the glyph
            drawing.new_path();
            drawing.rect(glyph_min_x, glyph_min_y, glyph_max_x, glyph_max_y);

            if active_color != Some(glyph_color) {
                // TODO: pick a better texture ID here, figure out why things hang when resizing with this in, figure out why we get mipmap errors too
                //drawing.copy_texture(texture_id, canvas::TextureId(1));
                //drawing.filter_texture(canvas::TextureId(1), canvas::TextureFilter::Tint(glyph_color));
                //active_color = Some(glyph_color);
            }

            drawing.fill_texture(texture_id, texture_min_x, texture_min_y, texture_max_x, texture_max_y);
            drawing.fill();
        }

        // Assuming the row begins at 0,0
        pos_y += row.rect.bottom();
    }

}

///
/// Draws a shape to a drawing vec
///
pub fn draw_shape(shape: &egui::Shape, drawing: &mut Vec<canvas::Draw>) {
    use canvas::{Draw, LayerId};
    use egui::{Shape};
    use Shape::*;

    match shape {
        Noop                            => { }
        Vec(shapes)                     => { shapes.iter().for_each(|shape| draw_shape(shape, drawing)); }
        Circle(circle)                  => { drawing.new_path(); drawing.circle(circle.center.x, circle.center.y, circle.radius); draw_fill(&circle.fill, drawing); draw_stroke(&circle.stroke, drawing); }
        Ellipse(_)                      => { /* TODO */ }
        LineSegment{points, stroke}     => { drawing.new_path(); drawing.move_to(points[0].x, points[0].y); points.iter().skip(1).for_each(|point| drawing.line_to(point.x, point.y)); draw_stroke(stroke, drawing); }
        Path(path_shape)                => { drawing.new_path(); drawing.move_to(path_shape.points[0].x, path_shape.points[0].y); path_shape.points.iter().skip(1).for_each(|point| drawing.line_to(point.x, point.y)); if path_shape.closed { drawing.close_path(); } draw_fill(&path_shape.fill, drawing); draw_path_stroke(&path_shape.stroke, drawing); }
        Rect(rect_shape)                => { draw_rect(rect_shape, drawing); }
        Text(text_shape)                => { draw_text(text_shape, drawing); }
        Mesh(mesh_shape)                => { todo!() }
        QuadraticBezier(_quad_bezier)   => { todo!() }
        CubicBezier(_cubic_bezier)      => { todo!() }
        Callback(_)                     => { /* Not supported */ }
    }
}
