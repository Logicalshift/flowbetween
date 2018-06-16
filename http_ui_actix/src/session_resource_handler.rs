use super::actix_session::*;

use flo_ui::*;
use flo_ui::session::*;
use flo_http_ui::*;

use actix_web::*;
use actix_web::Error;
use actix_web::http::*;
use actix_web::dev::{AsyncResult,Handler};
use futures::*;
use futures::future;

use std::str::*;
use std::sync::*;

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
        let controller_path = components[2..(components.len()-1)].iter().map(|element| element.to_string()).collect();
        let resource_name   = components[components.len()-1].to_string();

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
fn get_controller<CoreUi>(session: &HttpSession<CoreUi>, controller_path: Vec<String>) -> Option<Arc<Controller>>
where CoreUi: 'static+CoreUserInterface+Send+Sync {
    // Get the root controller
    let mut controller: Option<Arc<Controller>> = Some(session.ui().controller());

    controller
}

///
/// Produces a HTTP response for an image request
/// 
fn handle_image_request<Session: ActixSession>(req: &HttpRequest<Arc<Session>>, session: &HttpSession<Session::CoreUi>, controller_path: Vec<String>, image_name: String) -> impl Future<Item=HttpResponse, Error=Error> {
    // Try to fetch the controller at this path
    let controller = get_controller(session, controller_path);

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

        /*
        // Either return the image data, or not found
        if let Some(image) = image {
            // Return the image
            self.image_response(image)
        } else {
            // No image found
            Response::with(status::NotFound)
        }
        */

        unimplemented!()
    } else {
        // Controller not found
        future::ok(req.build_response(StatusCode::NOT_FOUND).body("Not found"))
    }
}

///
/// Handler for get requests for a session
/// 
pub fn session_resource_handler<Session: 'static+ActixSession>() -> impl Handler<Arc<Session>> {
    |req: HttpRequest<Arc<Session>>| {
        // The path is the tail of the request
        let path    = req.match_info().get("tail");
        let state   = Arc::clone(req.state());

        if let Some(path) = path {
            // Path is valid
            let resource = decode_url(path);

            if let Some(resource) = resource {
                // Got a valid resource
                let session = state.get_session(&resource.session_id);

                if let Some(session) = session {
                    // URL is in a valid format and the session could be found
                    match resource.resource_type {
                        ResourceType::Image     => AsyncResult::async(Box::new(handle_image_request(&req, &*session.lock().unwrap(), resource.controller_path, resource.resource_name))),
                        ResourceType::Canvas    => AsyncResult::ok(req.build_response(StatusCode::NOT_FOUND).body("Not found"))
                    }
                } else {
                    // URL is in a valid format but the session could not be found
                    AsyncResult::ok(req.build_response(StatusCode::NOT_FOUND).body("Not found"))
                }
            } else {
                // Resource URL was not in the expected format
                AsyncResult::ok(req.build_response(StatusCode::NOT_FOUND).body("Not found"))
            }
        } else {
            // No tail path was supplied (likely this handler is being called from the wrong place)
            AsyncResult::ok(req.build_response(StatusCode::NOT_FOUND).body("Not found"))
        }
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