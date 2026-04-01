use flo_ui::*;

use std::f64;

///
/// Given a colour strip (RGBA colours), turns it into a RGBA bitmap of
/// the specified size, which is both the width and the height.
///
pub fn rgba_data_for_color_wheel(rgba_color_strip: &[(u8, u8, u8, u8)], size: u32, inner_radius: u32, rotate_degrees: f64) -> Vec<u8> {
    // Some constants
    let pi              = f64::consts::PI;
    let two_pi          = pi * 2.0;
    let radius          = (size as f64)/2.0;
    let inner_radius    = inner_radius as f64;
    let rotate_radians  = (rotate_degrees/180.0)*pi;

    let ratio           = 1023.0/two_pi;
    let radius_squared  = radius*radius;
    let inner_squared   = inner_radius*inner_radius;

    let row_len         = (size*4) as usize;

    // Actual image data. Start with all transparent pixls
    let mut pixels      = vec![0; (size*size*4) as usize];

    // Build the pixels
    for y in 0..size {
        // Row in the image that we're editing
        let row_index   = (y * size * 4) as usize;
        let row         = &mut pixels[row_index..(row_index+row_len)];

        // Y position in the circle
        let y           = (y as f64)-radius;
        let y_squared   = y*y;

        for x in 0..(size as usize) {
            // RGBA pixels at this position
            let rgba = &mut row[x*4..(x*4+4)];

            // Work out the distance of this pixel from the center
            let x                   = (x as f64)-radius;
            let distance_squared    = x*x + y_squared;

            if distance_squared <= radius_squared && distance_squared >= inner_squared {
                // This pixel comes from the colour strip
                let angle   = f64::atan2(x, y) + pi + rotate_radians;
                let pos     = ((angle*ratio) as usize) % 1024;
                let colour  = &rgba_color_strip[pos];

                rgba[0]     = colour.0;
                rgba[1]     = colour.1;
                rgba[2]     = colour.2;
                rgba[3]     = colour.3;
            }
        }
    }

    pixels
}

///
/// Given a function that takes a value from 0-1 (representing the distance
/// around the colour wheel) and returns a pixel, generates a colour wheel.
///
pub fn rgba_data_for_wheel_fn<PixelFn: Fn(f64) -> (u8, u8, u8, u8)>(pixel: PixelFn, size: u32, inner_radius: u32, rotate_degrees: f64) -> Vec<u8> {
    let colour_strip: Vec<_> = (0..1024)
        .into_iter()
        .map(move |index| pixel((index as f64)/1024.0))
        .collect();

    rgba_data_for_color_wheel(&colour_strip, size, inner_radius, rotate_degrees)
}

///
/// Given a function that takes a value from 0-1 (representing the distance
/// around the colour wheel) and returns a pixel, generates a colour wheel.
///
pub fn image_for_wheel_fn<PixelFn: Fn(f64) -> (u8, u8, u8, u8)>(pixel: PixelFn, size: u32, inner_radius: u32, rotate_degrees: f64) -> Image {
    Image::png_from_rgba_data(&rgba_data_for_wheel_fn(pixel, size, inner_radius, rotate_degrees), size, size)
}
