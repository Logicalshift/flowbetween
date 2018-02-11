use std::str::*;
use std::sync::*;
use std::collections::*;

use super::event::*;
use super::update::*;
use super::session::*;
use super::htmlcontrol::*;
use super::canvas_body::*;
use super::session_state::*;

use ui::*;
use binding::Bound;

extern crate serde_json;

use iron::*;
use iron::mime::*;
use iron::method::*;
use iron::headers::*;
use iron::modifiers::*;
use mount::*;

use bodyparser::*;
use percent_encoding::*;

///
/// Handler that runs a particular UI through the HTTP interface
///
pub struct UiHandler<TSession: Session> {
    /// The sessions that are currently active for this handler
    active_sessions: Mutex<HashMap<String, (Arc<SessionState>, Arc<TSession>)>>,
}

impl<TSession: Session+'static> UiHandler<TSession> {
    ///
    /// Creates a new UI handler
    ///
    pub fn new() -> UiHandler<TSession> {
        UiHandler { 
            active_sessions: Mutex::new(HashMap::new()),  
        }
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
    pub fn new_session(&self) -> String {
        // Generate a new session
        let new_state   = Arc::new(SessionState::new());
        let new_session = Arc::new(TSession::start_new(new_state.clone()));

        // Set the initial UI state
        new_state.set_ui_tree(assemble_ui(new_session.clone()));
        new_state.watch_controller_viewmodel(new_session.clone());

        // Store in the list of active sessions
        let mut active_sessions = self.active_sessions.lock().unwrap();
        active_sessions.insert(String::from(new_state.id()), (new_state.clone(), new_session));

        // Result is the session ID
        String::from(new_state.id())
    }

    ///
    /// Generates a UI refresh response
    ///
    pub fn refresh_ui(&self, state: Arc<SessionState>, session: Arc<TSession>, response: &mut UiHandlerResponse, base_url: &str) {
        let base_path   = format!("{}/{}", base_url, state.id());
        let ui          = state.entire_ui_tree();
        let ui_html     = ui.to_html(&base_path);
        let viewmodel   = viewmodel_update_controller_tree(&*session);

        response.updates.push(Update::NewUserInterfaceHtml(ui_html.to_string(), ui.to_json(), viewmodel));
    }

    ///
    /// Sends the tick event to the specified controller
    /// 
    pub fn send_tick(controller: Arc<Controller>) {
        // Send to the subcontrollers
        let ui              = controller.ui().get();
        let subcontrollers  = ui.all_controllers();

        for subcontroller in subcontrollers {
            if let Some(subcontroller) = controller.get_subcontroller(&subcontroller) {
                Self::send_tick(subcontroller);
            }
        }

        // Send to this controlle
        controller.tick();
    }

    ///
    /// Dispatches an action to a controller
    ///
    pub fn dispatch_action(&self, session: Arc<TSession>, controller_path: &Vec<String>, action_name: &str, action_parameter: &ActionParameter) {
        // Follow the controller path
        let mut controller: Option<Arc<Controller>> = Some(session);

        for step in controller_path.iter() {
            controller = controller.map_or(None, |ctrl| ctrl.get_subcontroller(step));
        }

        // If there's a controller on this path, send the action to it
        match controller {
            Some(ref controller)    => controller.action(action_name, action_parameter),
            None                    => ()   // TODO: report error/warning - controller does not exist? Might be possible if the UI is catching up with the server state.
        };
    }

    ///
    /// Fills in a response structure for a request with no session
    ///
    fn handle_no_session(&self, response: &mut UiHandlerResponse, req: &UiHandlerRequest) {
        for event in req.events.iter() {
            match event.clone() {
                // When there is no session, we can request that one be created
                Event::NewSession => {
                    let session_id = self.new_session();
                    response.updates.push(Update::NewSession(session_id));
                },

                // For any other event, a session is required, so we add a 'missing session' notification to the response
                _ => response.updates.push(Update::MissingSession)
            }
        }
    }

    ///
    /// Dispatches a response structure to a session
    ///
    fn handle_with_session(&self, state: Arc<SessionState>, session: Arc<TSession>, response: &mut UiHandlerResponse, req: &UiHandlerRequest, base_url: &str) {
        use Event::*;
        let base_path = format!("{}/{}", base_url, state.id());

        // Cache the UI state before the event is processed
        let ui_before_event = state.entire_ui_tree();

        // Dispatch the events
        for event in req.events.iter() {
            match event.clone() {
                // Requesting a new session when there already is one is sort of pointless, but we allow it
                NewSession => {
                    let session_id = self.new_session();
                    response.updates.push(Update::NewSession(session_id));
                },

                // Refreshing the UI generates a new set of HTML from the abstract UI representation
                UiRefresh => self.refresh_ui(state.clone(), session.clone(), response, base_url),

                // Ticks do nothing
                Tick => (),

                // Actions are dispatched to the appropriate controller
                Action(ref controller_path, ref action, ref action_parameter) => self.dispatch_action(session.clone(), controller_path, action, action_parameter)
            }
        }

        // Dispatch a tick
        Self::send_tick(session.clone());

        // If the UI has changed, then add a HTML update to the response
        // TODO: if we're handling requests in parallel we actually need to diff against the UI state in whatever the most recent known state sent was rather than the state at the start
        let ui_after_event  = state.entire_ui_tree();
        let ui_differences  = diff_tree(&ui_before_event, &ui_after_event);

        if ui_differences.len() > 0 {
            // Turn the control differences into HTML differences
            let updates: Vec<HtmlDiff> = ui_differences.into_iter()
                .map(|ui_diff| {
                    let address         = ui_diff.address().clone();
                    let replacement     = ui_diff.replacement();
                    let controller_path = html_controller_path_for_address(&ui_after_event, &address);
                    let html            = replacement.to_html_subcomponent(&base_path, &controller_path);

                    HtmlDiff::new(address, replacement, html.to_string())
                })
                .collect();

            // Add the new update to the response
            response.updates.push(Update::UpdateHtml(updates));
        }

        // If the viewmodel has changerd, these changes are next
        let viewmodel_differences = state.cycle_viewmodel_watch();
        if viewmodel_differences.len() > 0 {
            response.updates.push(Update::UpdateViewModel(viewmodel_differences));
        }

        // Add in any canvas updates
        let canvas_differences = state.latest_canvas_updates();
        if canvas_differences.len() > 0 {
            response.updates.push(Update::UpdateCanvas(canvas_differences));
        }
    }

    ///
    /// Handles a UI handler request
    ///
    pub fn handle_ui_request(&self, req: &UiHandlerRequest, base_url: &str) -> Response {
        // The response that we'll return for this request
        let mut response = UiHandlerResponse { updates: vec![] };

        // Dispatch depending on whether or not this request corresponds to an active session
        match req.session_id {
            None                    => self.handle_no_session(&mut response, req),
            Some(ref session_id)    => {
                // Try to fetch the session for this ID
                let mut active_sessions = self.active_sessions.lock().unwrap();
                let session             = active_sessions.get_mut(session_id);

                // If the session ID is not presently registered, then we proceed as if the session is missing 
                match session {
                    Some(&mut (ref session_state, ref session)) => 
                        self.handle_with_session(session_state.clone(), session.clone(), &mut response, req, base_url),
                    _ => 
                        self.handle_no_session(&mut response, req)
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
    fn decode_controller_path(&self, session: Arc<TSession>, relative_url: Url) -> Option<(Arc<Controller>, String)> {
        // Not found if the path is empty
        if relative_url.path().len() == 0 {
            return None;
        }

        let path = relative_url.path();

        // The first part of the path indicates the controller
        let mut controller: Option<Arc<Controller>> = Some(session);

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
    pub fn handle_canvas_get(&self, session: Arc<TSession>, relative_url: Url) -> Response {
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
    pub fn handle_image_get(&self, session: Arc<TSession>, relative_url: Url) -> Response {
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

        let active_sessions = self.active_sessions.lock().unwrap();

        // Try to retrieve the session
        let session_id      = req.url.path()[0];
        let resource_type   = req.url.path()[1];

        let session         = active_sessions.get(session_id);

        if let Some(&(ref _session_state, ref session)) = session {
            let remaining_path  = req.url.path()[2..].join("/");
            let mut partial_url = req.url.clone();

            partial_url.as_mut().set_path(&remaining_path);

            // Action depends on the resource type
            match resource_type {
                // 'i' is shorthand for 'image'
                "i"     => self.handle_image_get(session.clone(), partial_url),
                "c"     => self.handle_canvas_get(session.clone(), partial_url),

                _       => Response::with((status::NotFound))
            }
        } else {
            // Session not found
            Response::with((status::NotFound))
        }
    }
}

///
/// Structure of a request sent to the UI handler
///
#[derive(Clone, Serialize, Deserialize)]
pub struct UiHandlerRequest {
    /// The session ID, if there is one
    session_id: Option<String>,

    /// The events that the UI wishes to report with this request
    events: Vec<Event>
}

///
/// Structure of a UI handler response
///
#[derive(Clone, Serialize, Deserialize)]
pub struct UiHandlerResponse {
    /// Updates generated for this request
    updates: Vec<Update>
}

impl<TSession: Session+'static> Handler for UiHandler<TSession> {
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
                        Ok(Some(request))   => Ok(self.handle_ui_request(&request, &base_url)),
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