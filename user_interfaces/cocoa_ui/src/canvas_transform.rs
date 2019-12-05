use super::core_graphics_ffi::*;

///
/// Computes the identity transform for this canvas
///
pub fn canvas_identity_transform(viewport_origin: (f64, f64), canvas_size: (f64, f64)) -> CGAffineTransform {
    unsafe {
        let (origin_x, origin_y)    = viewport_origin;
        let (width, height)         = canvas_size;
        let scale                   = (height as CGFloat)/2.0;

        let transform = CGAffineTransformIdentity;
        let transform = CGAffineTransformTranslate(transform, -origin_x as CGFloat, -origin_y as CGFloat);
        let transform = CGAffineTransformTranslate(transform, (width as CGFloat)/2.0, (height as CGFloat)/2.0);
        let transform = CGAffineTransformScale(transform, scale, -scale);

        transform
    }
}

///
/// Computes a matrix to be appended to the identity transform that will set the height of the canvas
///
pub fn canvas_height_transform(height: f64) -> CGAffineTransform {
    unsafe {
        let mut ratio_x = 2.0/height;
        let ratio_y     = ratio_x;

        if height < 0.0 {
            ratio_x = -ratio_x;
        }

        let result = CGAffineTransformIdentity;
        let result = CGAffineTransformScale(result, ratio_x as CGFloat, ratio_y as CGFloat);

        result
    }
}

///
/// Retrieves the transformation needed to move the center of the canvas to the specified point
///
pub fn canvas_center_transform(viewport_origin: (f64, f64), canvas_size: (f64, f64), current_transform: CGAffineTransform, minx: f64, miny: f64, maxx: f64, maxy: f64) -> CGAffineTransform {
    unsafe {
        let (origin_x, origin_y)        = viewport_origin;
        let (pixel_width, pixel_height) = canvas_size;
        let current_transform           = current_transform;

        // Get the current scaling of this canvas
        let mut xscale = (current_transform.a*current_transform.a + current_transform.b*current_transform.b).sqrt();
        let mut yscale = (current_transform.c*current_transform.c + current_transform.d*current_transform.d).sqrt();
        if xscale == 0.0 { xscale = 1.0; }
        if yscale == 0.0 { yscale = 1.0; }

        // Current X, Y coordinates (centered)
        let cur_x = (current_transform.tx-(pixel_width/2.0))/xscale;
        let cur_y = (current_transform.ty-(pixel_height/2.0))/yscale;

        // New center coordinates
        let center_x = (minx+maxx)/2.0;
        let center_y = (miny+maxy)/2.0;

        // Compute the offsets and transform the canvas
        let x_offset = cur_x - center_x;
        let y_offset = cur_y - center_y;

        let x_offset = x_offset + origin_x/xscale;
        let y_offset = y_offset + origin_y/xscale;

        // Generate the result matrix
        let result = CGAffineTransformIdentity;
        let result = CGAffineTransformTranslate(result, x_offset as CGFloat, y_offset as CGFloat);
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn identity_transform_bottom_left_corner_square() {
        unsafe {
            // For the identity transform, y = -1.0 should be at the bottom of the canvas
            let identity    = canvas_identity_transform((0.0, 0.0), (1080.0, 1080.0));
            let bottom_left = CGPointApplyAffineTransform(CGPoint { x: -1.0, y: -1.0 }, identity);

            assert!((bottom_left.y-1080.0).abs() < 0.01);
            assert!((bottom_left.x-0.0).abs() < 0.01);
        }
    }

    #[test]
    fn identity_transform_top_right_corner_square() {
        unsafe {
            // For the identity transform, y = 1.0 should be at the top of the canvas
            let identity    = canvas_identity_transform((0.0, 0.0), (1080.0, 1080.0));
            let top_right   = CGPointApplyAffineTransform(CGPoint { x: 1.0, y: 1.0 }, identity);

            assert!((top_right.y-0.0).abs() < 0.01);
            assert!((top_right.x-1080.0).abs() < 0.01);
        }
    }

    #[test]
    fn identity_transform_rectangle() {
        unsafe {
            // For the identity transform, y = 1.0 should be at the top of the canvas
            let identity        = canvas_identity_transform((0.0, 0.0), (1920.0, 1080.0));
            let middle_bottom   = CGPointApplyAffineTransform(CGPoint { x: 0.0, y: -1.0 }, identity);

            assert!((middle_bottom.y-1080.0).abs() < 0.01);
            assert!((middle_bottom.x-960.0).abs() < 0.01);
        }
    }

    #[test]
    fn identity_transform_viewport() {
        unsafe {
            // For the identity transform, y = 1.0 should be at the top of the canvas
            let identity        = canvas_identity_transform((256.0, 256.0), (1920.0, 1080.0));
            let middle_bottom   = CGPointApplyAffineTransform(CGPoint { x: 0.0, y: -1.0 }, identity);

            assert!((middle_bottom.y-824.0).abs() < 0.01);
            assert!((middle_bottom.x-704.0).abs() < 0.01);
        }
    }

    #[test]
    fn height_transform_square() {
        unsafe {
            // For the identity transform, y = -1.0 should be at the bottom of the canvas
            let identity    = canvas_identity_transform((0.0, 0.0), (1080.0, 1080.0));
            let height      = canvas_height_transform(100.0);
            let combined    = CGAffineTransformConcat(height, identity);
            let bottom_left = CGPointApplyAffineTransform(CGPoint { x: -50.0, y: -50.0 }, combined);

            println!("{:?}", bottom_left);

            assert!((bottom_left.y-1080.0).abs() < 0.01);
            assert!((bottom_left.x-0.0).abs() < 0.01);
        }
    }
}
