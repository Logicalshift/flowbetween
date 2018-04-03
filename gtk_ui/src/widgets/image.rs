use flo_ui::*;

use gio;
use glib;
use gdk_pixbuf;

///
/// Creates a pixbuf from a FlowBetween image resource
/// 
pub fn pixbuf_from_image(image: Resource<Image>) -> gdk_pixbuf::Pixbuf {
    use self::Image::*;

    match &*image {
        &Png(ref image_data) => pixbuf_from_png(&**image_data),
        &Svg(ref image_data) => pixbuf_from_svg(&**image_data)
    }
}

///
/// Creates some glib bytes from an image data object
/// 
fn bytes_from_data(image_data: &ImageData) -> glib::Bytes {
    // Read the image data out into a byte buffer
    let mut data = vec![];
    image_data.read()
        .read_to_end(&mut data)
        .unwrap();

    // Turn into a glib Bytes object
    glib::Bytes::from_owned(data)
}

///
/// Creates a pixbuf from PNG data
/// 
fn pixbuf_from_png(image_data: &ImageData) -> gdk_pixbuf::Pixbuf {
    let bytes           = bytes_from_data(image_data);
    let input_stream    = gio::MemoryInputStream::new_from_bytes(&bytes);

    gdk_pixbuf::Pixbuf::new_from_stream(&input_stream, None).unwrap()
}

///
/// Creates a pixbuf from SVG data
/// 
fn pixbuf_from_svg(image_data: &ImageData) -> gdk_pixbuf::Pixbuf {
    let bytes           = bytes_from_data(image_data);
    let input_stream    = gio::MemoryInputStream::new_from_bytes(&bytes);

    gdk_pixbuf::Pixbuf::new_from_stream(&input_stream, None).unwrap()
}
