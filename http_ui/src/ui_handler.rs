use super::event::*;
use super::update::*;
use super::sessions::*;
use super::canvas_body::*;
use super::http_session::*;
use super::http_controller::*;

use ui::*;
use ui::session::*;

use iron::*;
use iron::mime::*;
use iron::method::*;
use iron::headers::*;
use iron::modifiers::*;
use mount::*;

use serde_json;
use bodyparser::*;
use futures::executor;
use percent_encoding::*;

use std::str::*;
use std::sync::*;

///
/// Handles creating and maintainng HTTP sessions
/// 
pub struct UiHandler<CoreController: HttpController> {
    /// The sessions that this handler has active
    sessions: Arc<WebSessions<CoreController>>,

    /// If we want to indicate new sessions should use WebSockets, this is the port for the connection
    websocket_port: Option<u32>
}

impl<CoreController: HttpController+'static> UiHandler<CoreController> {
    ///
    /// Creates a new UI handler
    /// 
    pub fn new() -> UiHandler<CoreController> {
        UiHandler {
            sessions:       Arc::new(WebSessions::new()),
            websocket_port: None
        }
    }

    ///
    /// Creates a websocket handler that will provide websockets for a pre-set
    /// set of sessions
    /// 
    pub fn from_sessions(sessions: Arc<WebSessions<CoreController>>) -> UiHandler<CoreController> {
        UiHandler { 
            sessions:       sessions,
            websocket_port: None
        }
    }

    ///
    /// Sets the websocket port that this should report when creating a new session
    /// 
    pub fn set_websocket_port(&mut self, port: u32) {
        self.websocket_port = Some(port);
    }

    ///
    /// Returns the base URL for a request
    ///
    fn base_url(req: &Request) -> Url {
        // Get the original URL for this request
        let original_url = req.extensions.get::<OriginalUrl>()
            .map(|url| url.clone())
            .unwrap_or(Url::parse("http://localhost/").unwrap());
        
        // Also need the request url
        let request_url     = req.url.clone();

        // Request URL path will be the last part of the original URL: remove enough parts that 
        let original_path       = original_url.path();
        let request_path        = request_url.path();

        let original_path_len   = original_path.len();
        let request_path_len    = {
            if request_path.len() == 1 && request_path[0] == "" {
                0
            } else if request_path.len() > original_path.len() {
                0
            } else {
                request_path.len()
            }
        };

        let base_path           = original_url.path()[0..(original_path_len-request_path_len)].join("/");

        let mut base_url = original_url.clone();
        base_url.as_mut().set_path(&base_path);

        base_url
    }

    ///
    /// Creates a new session and session state, returning the ID
    ///
    pub fn new_session(&self, base_url: &str) -> String {
        // Create the session controller
        let session_controller  = CoreController::start_new();

        // Store in the list of active sessions
        let session_id = self.sessions.new_session(session_controller, base_url);

        // Result is the session ID
        session_id
    }

    ///
    /// Fills in a response structure for a request with no session
    ///
    fn handle_no_session(&self, base_url: &str, response: &mut UiHandlerResponse, req: &UiHandlerRequest) {
        for event in req.events.iter() {
            match event.clone() {
                // When there is no session, we can request that one be created
                Event::NewSession => {
                    // Start a new session
                    let session_id = self.new_session(base_url);

                    // Indicate the session ID
                    response.updates.push(Update::NewSession(session_id));

                    // If we support websockets, indicate where the websocket can be found
                    if let Some(websocket_port) = self.websocket_port {
                        response.updates.push(Update::WebsocketPort(websocket_port));
                    }
                },

                // For any other event, a session is required, so we add a 'missing session' notification to the response
                _ => response.updates.push(Update::MissingSession)
            }
        }
    }

    ///
    /// Sends a request to a session
    /// 
    fn handle_with_session(&self, session: &mut HttpSession<UiSession<CoreController>>, response: &mut UiHandlerResponse, mut req: UiHandlerRequest) {
        // Always finish with a tick event
        req.events.push(Event::Tick);

        // Send the events to the session
        let handle_result = session.send_events(req.events);

        // Wait for the response
        let update_results = executor::spawn(handle_result).wait_future().unwrap();

        // Add to the response
        response.updates.extend(update_results);
    }

    ///
    /// Handles a UI handler request
    ///
    pub fn handle_ui_request(&self, req: UiHandlerRequest, base_url: &str) -> Response {
        // The response that we'll return for this request
        let mut response    = UiHandlerResponse { updates: vec![] };
        let session_id      = req.session_id.clone();

        // Dispatch depending on whether or not this request corresponds to an active session
        match session_id {
            None                    => self.handle_no_session(base_url, &mut response, &req),
            Some(ref session_id)    => {
                // Try to fetch the session for this ID
                let session = self.sessions.get_session(session_id);

                // If the session ID is not presently registered, then we proceed as if the session is missing 
                match session {
                    Some(session) => {
                        let mut session = session.lock().unwrap();
                        self.handle_with_session(&mut session, &mut response, req)
                    },
                    _ => 
                        self.handle_no_session(base_url, &mut response, &req)
                }
            }
        };

        // Generate the final response
        Response::with((
            status::Ok,
            Header(ContentType::json()),
            serde_json::to_string(&response).unwrap()
        ))
    }

    ///
    /// Returns the controller and the resource name for a URL containing a controller/resource path
    ///
    fn decode_controller_path(&self, session: &HttpSession<UiSession<CoreController>>, relative_url: Url) -> Option<(Arc<Controller>, String)> {
        // Not found if the path is empty
        if relative_url.path().len() == 0 {
            return None;
        }

        let path = relative_url.path();

        // The first part of the path indicates the controller
        let mut controller: Option<Arc<Controller>> = Some((**session.ui()).clone());

        for path_component in 0..(path.len()-1) {
            let next_controller_name = &*percent_decode(path[path_component].as_bytes())
                .decode_utf8()
                .map(|cow| cow.into_owned())
                .unwrap_or(String::from(path[path_component]));
            controller = controller.map_or(None, move |controller| controller.get_subcontroller(next_controller_name));
        }

        // Final component is the resource name (or id)
        let resource_name = String::from(*path.last().unwrap());

        controller.map(move |controller| (controller, resource_name))
    }

    ///
    /// Generates a response for a canvas
    ///
    fn canvas_response(&self, canvas: &Resource<BindingCanvas>) -> Response {
        let mut response = Response::with((
            status::Ok,
            Header(ContentType("application/flocanvas; charset=utf-8".parse::<Mime>().unwrap()))
        ));
        response.body = Some(Box::new(CanvasBody::new(canvas)));
        response
    }

    ///
    /// Attempts to retrieve a canvas from the session
    ///
    pub fn handle_canvas_get(&self, session: &HttpSession<UiSession<CoreController>>, relative_url: Url) -> Response {
        if let Some((controller, canvas_name)) = self.decode_controller_path(session, relative_url) {
            let canvas_resources    = controller.get_canvas_resources();
            let canvas              = canvas_resources.map_or(None, |resources| {
                if let Ok(id) = u32::from_str(&canvas_name) {
                    resources.get_resource_with_id(id)
                } else {
                    resources.get_named_resource(&canvas_name)
                }
            });

            if let Some(canvas) = canvas {
                self.canvas_response(&canvas)
            } else {
                Response::with((status::NotFound))
            }
        } else {
            // Not found
            Response::with((status::NotFound))
        }
    }

    ///
    /// Generates a HTTP response containing image data
    /// 
    fn image_response(&self, image: Resource<Image>) -> Response {
        match *image {
            Image::Png(ref data) => {
                let mut response = Response::with((
                    status::Ok,
                    Header(ContentType::png())
                ));
                response.body = Some(Box::new(data.read()));
                response
            },

            Image::Svg(ref data) => {
                let mut response = Response::with((
                    status::Ok,
                    Header(ContentType("image/svg+xml; charset=utf-8".parse::<Mime>().unwrap()))
                ));
                response.body = Some(Box::new(data.read()));
                response
            }
        }
    }

    ///
    /// Attempts to retrieve an image from the session
    ///
    pub fn handle_image_get(&self, session: &HttpSession<UiSession<CoreController>>, relative_url: Url) -> Response {
        if let Some((controller, image_name)) = self.decode_controller_path(session, relative_url) {
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
                // Return the image
                self.image_response(image)
            } else {
                // No image found
                Response::with((status::NotFound))
            }
        } else {
            return Response::with((status::NotFound));
        }
    }

    ///
    /// Handles a get resources request
    /// 
    pub fn handle_resource_request(&self, req: &mut Request) -> Response {
        if req.url.path().len() < 2 {
            // Path should be session_id/resource_type
            return Response::with((status::NotFound));
        }


        // Try to retrieve the session
        let session_id      = req.url.path()[0];
        let resource_type   = req.url.path()[1];

        let session         = self.sessions.get_session(session_id);

        if let Some(session) = session {
            let session         = session.lock().unwrap();
            let remaining_path  = req.url.path()[2..].join("/");
            let mut partial_url = req.url.clone();

            partial_url.as_mut().set_path(&remaining_path);

            // Action depends on the resource type
            match resource_type {
                // 'i' is shorthand for 'image'
                "i"     => self.handle_image_get(&*session, partial_url),
                "c"     => self.handle_canvas_get(&*session, partial_url),

                _       => Response::with((status::NotFound))
            }
        } else {
            // Session not found
            Response::with((status::NotFound))
        }
    }
}

impl<CoreController: HttpController+'static> Handler for UiHandler<CoreController> {
    ///
    /// Handles a request for a UI session (or creates new sessions)
    ///
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        match req.method {
            Method::Post => {
                let is_json         = match req.headers.get() { Some(&ContentType(Mime(TopLevel::Application, SubLevel::Json, _))) => true, _ => false };
                let mut base_url    = Self::base_url(req).path().join("/");

                if base_url.chars().nth(0) != Some('/') {
                    base_url.insert(0, '/');
                }

                if !is_json {
                    // Must be a JSON POST request
                    Ok(Response::with((status::BadRequest)))
                } else {
                    // Parse the request
                    let request = req.get::<Struct<UiHandlerRequest>>();

                    match request {
                        Ok(Some(request))   => Ok(self.handle_ui_request(request, &base_url)),
                        Ok(None)            => Ok(Response::with((status::BadRequest))),
                        Err(_)              => Ok(Response::with((status::BadRequest)))
                    }
                }
            },

            Method::Get => {
                // Resource fetch
                Ok(self.handle_resource_request(req))
            },

            _ => {
                // Unsupported method
                Ok(Response::with((status::BadRequest)))
            }
        }
    }
}

///
/// Structure of a request sent to the UI handler
///
#[derive(Clone, Serialize, Deserialize)]
pub struct UiHandlerRequest {
    /// The session ID, if there is one
    pub session_id: Option<String>,

    /// The events that the UI wishes to report with this request
    pub events: Vec<Event>
}

///
/// Structure of a UI handler response
///
#[derive(Clone, Serialize, Deserialize)]
pub struct UiHandlerResponse {
    /// Updates generated for this request
    pub updates: Vec<Update>
}
