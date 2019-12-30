use super::*;

use ::png;

///
/// Creates a PNG image in memory for an RGBA buffer
///
pub fn png_data_for_rgba(rgba: &[u8], width: u32, height: u32) -> InMemoryImageData {
    // Create the buffer to write the PNG data to
    let mut png_data: Vec<u8> = vec![];

    {
        // Create an encoder that will write to this buffer
        let mut png_encoder = png::Encoder::new(&mut png_data, width, height);
        png_encoder.set_color(png::ColorType::RGBA);
        png_encoder.set_depth(png::BitDepth::Eight);

        // Write the header
        let mut png_writer = png_encoder.write_header().unwrap();

        // Write the image data
        png_writer.write_image_data(rgba).unwrap();
    }

    // Generate the image data object for the final PNG
    InMemoryImageData::from(png_data)
}
