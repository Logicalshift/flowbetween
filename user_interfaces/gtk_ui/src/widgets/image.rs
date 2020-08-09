use flo_ui::*;

use gio;
use glib;
use gdk_pixbuf;
use gtk;
use gtk::prelude::*;

///
/// Creates an image widget from an image resource
///
pub fn image_from_image(image: Resource<Image>) -> gtk::Image {
    // Create a new image widget
    let pixbuf          = pixbuf_from_image(image);
    let image_widget    = gtk::Image::new();

    // GTK can't auto-scale images, so we'll do that ourselves
    image_widget.connect_size_allocate(move |image, allocation| {
        let image = image.clone();
        if let Some(image) = image.dynamic_cast::<gtk::Image>().ok() {
            // Work out the scale ratio for the image (so we fit it into the control but keep the aspect ratio)
            let (image_width, image_height)     = (pixbuf.get_width() as f64, pixbuf.get_height() as f64);
            let (target_width, target_height)   = (allocation.width as f64, allocation.height as f64);
            let (ratio_w, ratio_h)              = (target_width/image_width, target_height/image_height);
            let ratio                           = ratio_w.min(ratio_h);

            // Create a scaled image with that ratio
            let (new_width, new_height)         = (image_width * ratio, image_height * ratio);
            let (new_width, new_height)         = (new_width.floor() as i32, new_height.floor() as i32);

            // Scale the image to fit
            let scaled = pixbuf.scale_simple(new_width, new_height, gdk_pixbuf::InterpType::Bilinear);
            scaled.map(|scaled| image.set_from_pixbuf(Some(&scaled)));
        }
    });

    image_widget
}

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
fn bytes_from_data(image_data: &dyn ImageData) -> glib::Bytes {
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
pub fn pixbuf_from_png(image_data: &dyn ImageData) -> gdk_pixbuf::Pixbuf {
    let bytes           = bytes_from_data(image_data);
    let input_stream    = gio::MemoryInputStream::from_bytes(&bytes);
    let not_cancellable: Option<gio::Cancellable> = None;

    gdk_pixbuf::Pixbuf::from_stream(&input_stream, not_cancellable.as_ref()).unwrap()
}

///
/// Creates a pixbuf from SVG data
///
pub fn pixbuf_from_svg(image_data: &dyn ImageData) -> gdk_pixbuf::Pixbuf {
    let bytes           = bytes_from_data(image_data);
    let input_stream    = gio::MemoryInputStream::from_bytes(&bytes);
    let not_cancellable: Option<gio::Cancellable> = None;

    gdk_pixbuf::Pixbuf::from_stream(&input_stream, not_cancellable.as_ref()).unwrap()
}
