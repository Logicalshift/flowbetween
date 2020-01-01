use super::actix_session::*;

use flo_ui::*;
use flo_ui::session::*;
use flo_canvas::*;
use flo_http_ui::*;
use flo_logging::*;

use actix_web::*;
use actix_web::Error;
use futures::*;
use futures::future;
use futures::stream;
use futures::future::{LocalBoxFuture};
use bytes::Bytes;
use percent_encoding::*;

use std::str::*;
use std::sync::*;

lazy_static! {
    /// The standard log for the resource handler
    static ref RESOURCE_HANDLER_LOG: LogPublisher = LogPublisher::new(module_path!());
}

///
/// Types of resource that can be retrieved statically
///
#[derive(Debug, PartialEq, Clone, Copy)]
enum ResourceType {
    Image,
    Canvas
}

impl ResourceType {
    ///
    /// Creates a resource type from the resource type element of the path
    ///
    fn from_path_element(item: &str) -> Option<ResourceType> {
        match item {
            "i" => Some(ResourceType::Image),
            "c" => Some(ResourceType::Canvas),

            _   => None
        }
    }
}

///
/// Struct representing the decoded meaning of a resource URL
///
#[derive(Debug, PartialEq, Clone)]
struct ResourceUrl {
    session_id:         String,
    resource_type:      ResourceType,
    controller_path:    Vec<String>,
    resource_name:      String
}

///
/// Decodes a URL into a resource URL
///
fn decode_url(path: &str) -> Option<ResourceUrl> {
    // Remove any '/' at the start of the path
    let path = if path.chars().nth(0) == Some('/') {
        path[1..].to_string()
    } else {
        path.to_string()
    };

    // Split up the components using the URL path separator
    let components: Vec<_> = path.split('/').collect();

    // Must be at least a session ID, resource type and resource name
    if components.len() < 3 {
        // Not enough components for a valid path
        None
    } else if let Some(resource_type) = ResourceType::from_path_element(components[1]) {
        // Enough components and we have a valid resource type
        let session_id      = components[0].to_string();
        let controller_path = components[2..(components.len()-1)].iter().map(|element| element.to_string());
        let resource_name   = components[components.len()-1].to_string();

        let controller_path = controller_path.map(|element| percent_decode(element.as_bytes())
            .decode_utf8().ok()
            .map(|decoded| String::from(decoded))
            .unwrap_or_else(|| element.clone()));
        let controller_path = controller_path.collect();

        Some(ResourceUrl {
            session_id,
            resource_type,
            controller_path,
            resource_name
        })
    } else {
        // Not a valid resource type
        None
    }
}

///
/// Finds the controller ont he specified path in the given session
///
fn get_controller<CoreUi>(session: &HttpSession<CoreUi>, controller_path: Vec<String>) -> Option<Arc<dyn Controller>>
where CoreUi: 'static+CoreUserInterface+Send+Sync {
    // Get the root controller
    let mut controller: Option<Arc<dyn Controller>> = Some(session.ui().controller());

    if controller.is_none() { println!("Root controller missing"); }

    // Try to navigate each section of the path
    for controller_name in controller_path {
        controller = controller.and_then(|controller| controller.get_subcontroller(&controller_name));
        if controller.is_none() { println!("Controller lookup failed at {:?}", controller_name); }
    }

    controller
}

///
/// Produces a HTTP response for an image request
///
fn handle_image_request<Session: ActixSession>(_req: HttpRequest, session: &HttpSession<Session::CoreUi>, controller_path: Vec<String>, image_name: String) -> impl Future<Output=Result<HttpResponse, Error>> {
    // Try to fetch the controller at this path
    let controller = get_controller(session, controller_path.clone());

    if let Some(controller) = controller {
        // Final component is the image name (or id)
        let image_resources = controller.get_image_resources();
        let image           = image_resources.map_or(None, |resources| {
            if let Ok(id) = u32::from_str(&image_name) {
                resources.get_resource_with_id(id)
            } else {
                resources.get_named_resource(&image_name)
            }
        });

        // Either return the image data, or not found
        if let Some(image) = image {
            match &*image {
                Image::Png(data) => {
                    // PNG data
                    future::ok(HttpResponse::Ok()
                        .header(http::header::CONTENT_TYPE, "image/png")
                        .streaming(data.read_future().map(|bytes| -> Result<_, Error> { Ok(bytes) })))
                },

                Image::Svg(data) => {
                    // SVG data
                    future::ok(HttpResponse::Ok()
                        .header(http::header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")
                        .streaming(data.read_future().map(|bytes| -> Result<_, Error> { Ok(bytes) })))
                },
            }
        } else {
            // Image not found
            session.log().log((Level::Warn, format!("Image `{}` not found", image_name)));

            future::ok(HttpResponse::NotFound().body("Not found"))
        }
    } else {
        // Controller not found
        session.log().log((Level::Warn, format!("Controller `{:?}` not found while looking for an image", controller_path)));

        future::ok(HttpResponse::NotFound().body("Not found"))
    }
}

///
/// Produces a HTTP response for a canvas request
///
fn handle_canvas_request<Session: ActixSession>(_req: HttpRequest, session: &HttpSession<Session::CoreUi>, controller_path: Vec<String>, canvas_name: String) -> impl Future<Output=Result<HttpResponse, Error>> {
    // Try to fetch the controller at this path
    let controller = get_controller(session, controller_path.clone());

    if let Some(controller) = controller {
        // Final component is the canvas name
        let canvas_resources    = controller.get_canvas_resources();
        let canvas              = canvas_resources.map_or(None, |resources| {
            if let Ok(id) = u32::from_str(&canvas_name) {
                resources.get_resource_with_id(id)
            } else {
                resources.get_named_resource(&canvas_name)
            }
        });

        if let Some(canvas) = canvas {
            // Stream encoding the canvas
            let drawing         = canvas.get_drawing();

            let encoded_drawing = drawing.into_iter()
                .map(|cmd| {
                    let mut encoded = String::new();
                    cmd.encode_canvas(&mut encoded);
                    encoded.push('\n');

                    encoded
                })
                .map(|encoded| Bytes::copy_from_slice(encoded.as_bytes()));

            let encoded_drawing = stream::iter(encoded_drawing).map(|s| -> Result<_, Error> { Ok(s) });

            // Turn into a response
            future::ok(HttpResponse::Ok()
                .header(http::header::CONTENT_TYPE, "application/flocanvas; charset=utf-8")
                .streaming(encoded_drawing))
        } else {
            // Canvas not found
            session.log().log((Level::Warn, format!("Canvas `{}` not found", canvas_name)));

            future::ok(HttpResponse::NotFound().body("Not found"))
        }
    } else {
        // Controller not found
        session.log().log((Level::Warn, format!("While searching for a canvas: controller `{:?}` not found", controller_path)));

        future::ok(HttpResponse::NotFound().body("Not found"))
    }
}

///
/// Handler for get requests for a session
///
pub fn session_resource_handler<Session: 'static+ActixSession>(req: HttpRequest) -> LocalBoxFuture<'static, Result<HttpResponse, Error>> {
    // The path is the tail of the request
    let path    = req.match_info().get("tail").map(|s| String::from(s));
    let state   = req.app_data::<Arc<Session>>().cloned();
    let state   = state.expect("Valid flowbetween session");

    if let Some(path) = path {
        // Path is valid
        let resource = decode_url(&path);

        if let Some(resource) = resource {
            // Got a valid resource
            let session = state.get_session(&resource.session_id);

            if let Some(session) = session {
                // URL is in a valid format and the session could be found
                match resource.resource_type {
                    ResourceType::Image     => Box::pin(handle_image_request::<Session>(req, &*session.lock().unwrap(), resource.controller_path, resource.resource_name)),
                    ResourceType::Canvas    => Box::pin(handle_canvas_request::<Session>(req, &*session.lock().unwrap(), resource.controller_path, resource.resource_name))
                }
            } else {
                // URL is in a valid format but the session could not be found
                RESOURCE_HANDLER_LOG.log((Level::Warn, format!("Session `{}` not found", resource.session_id)));

                Box::pin(future::ok(HttpResponse::NotFound().body("Not found")))
            }
        } else {
            // Resource URL was not in the expected format
            RESOURCE_HANDLER_LOG.log((Level::Warn, format!("Path `{}` was not in the expected format", path)));

            Box::pin(future::ok(HttpResponse::NotFound().body("Not found")))
        }
    } else {
        // No tail path was supplied (likely this handler is being called from the wrong place)
        RESOURCE_HANDLER_LOG.log((Level::Warn, format!("Missing tail path")));

        Box::pin(future::ok(HttpResponse::NotFound().body("Not found")))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_decode_valid_uri() {
        let decoded = decode_url("/some-session-id/i/controller1/controller2/image.png");

        assert!(decoded.is_some());

        let decoded = decoded.unwrap();
        assert!(decoded.session_id == "some-session-id".to_string());
        assert!(decoded.resource_type == ResourceType::Image);
        assert!(decoded.controller_path == vec!["controller1".to_string(), "controller2".to_string()]);
        assert!(decoded.resource_name == "image.png".to_string());
    }
}
