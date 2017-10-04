// FlowBetween
function flowbetween() {
    /// The ID of the running session
    let running_session_id = '';

    /// URL where the flowbetween session resides
    let target_url = '/flowbetween/session';

    let utf8 = new TextEncoder('utf-8');

    ///
    /// ===== LOGGING
    ///

    ///
    /// Note something
    ///
    let note = (msg) => {
        console.log('%c==> ' + msg, 'font-family: monospace; font-size: 80%; color: gray;');
    }

    ///
    /// Display a warning
    ///
    let warn = function() {
        console.warn.apply(console, arguments);
    }
    
    ///
    /// Display an error
    ///
    let error = function() {
        console.error.apply(console, arguments);
    }

    ///
    /// ===== SENDING REQUESTS
    ///

    ///
    /// Returns a promise that pauses for a certain time
    ///
    let pause = (seconds) => {
        return new Promise((resolve) => {
            if (seconds <= 0) {
                resolve();
            } else {
                setTimeout(() => resolve(), seconds * 1000.0);
            }
        });
    }

    ///
    /// Performs an XmlHttpRequest to a particular url with a JSON
    /// object, returning a promise.
    ///
    let xhr = (obj, url, method) => {
        obj     = obj       || {};
        url     = url       || target_url;
        method  = method    || 'POST';

        let encoding    = JSON.stringify(obj);
        
        return new Promise((resolve, reject) => {
            // Prepare the request
            let req         = new XMLHttpRequest();

            req.open(method, url);
            req.setRequestHeader('Content-Type', 'application/json; charset=UTF-8');
            
            // Completing the request completes the promise
            req.addEventListener('load', function() {
                let evt = this;
                if (evt.status < 200 || evt.status > 299) {
                    // Server error
                    reject('Server returned ' + evt.status);
                } else {
                    // Successful response
                    resolve(evt);
                }
            });
            req.addEventListener('error', function() {
                let evt = this;
                error(evt);
                reject(evt);
            });

            // Send the request
            req.send(utf8.encode(encoding));
        });
    }

    /// Sends a POST request
    let http_post   = (obj, url) => xhr(obj, url, 'POST');

    /// Sets a GET request
    let http_get    = (obj, url) => xhr(obj, url, 'GET');

    /// Converts a XMLHttpRequest to a response object
    let response_to_object = (xmlRequest) => {
        return new Promise((resolve, reject) => {
            // Must be a JSON response
            if (!xmlRequest.getResponseHeader('Content-Type').includes('application/json')) {
                // This request only supports JSON
                reject('Server did not return a JSON response');
            } else {
                // Parse the response to generate the result
                resolve(JSON.parse(xmlRequest.response));
            }
        });
    }

    ///
    /// Retries an operation if it fails
    ///
    let retry       = (start_op, retrying_callback) => {
        return new Promise((resolve, reject) => {
            // These are the times we wait between retrying
            let timeouts    = [ 0, 1, 2, 5, 30 ];

            // This actually runs a try
            let run_try     = (pass) => {
                return pause(timeouts[pass])
                    .then(() => start_op())
                    .catch((reason) => {
                        // Notify the callback the first time we do a retry
                        if (pass === 0 && retrying_callback) {
                            retrying_callback();
                        }

                        // Either stop retrying or try the next pass
                        let next_pass = pass + 1;
                        if (next_pass >= timeouts.length) {
                            reject(reason);
                        } else {
                            return run_try(next_pass);
                        }
                    })
                    .then(result => resolve(result));
            }
            
            // Run the first try
            run_try(0);
        });
    };

    ///
    /// ===== DOM MANIPULATION
    ///

    ///
    /// Fetches the root of the UI
    ///
    let get_root = () => {
        return document.getElementById('root');
    }

    ///
    /// Fetches the attributes for a control node
    ///
    let get_attributes = (control_data) => {
        // Fetch the raw attributes
        let attributes = control_data.attributes;

        // all() can be used to read all of the attributes
        let all = () => attributes;

        // get_attr(name) will retrieve the attribute with the given name (or null if it does not exist)
        let get_attr = (name) => {
            for (let attribute_index=0; attribute_index < attributes.length; ++attribute_index) {
                let attr        = attributes[attribute_index];
                let attr_name   = Object.keys(attr)[0];

                if (attr_name === name) {
                    return attr[attr_name];
                }
            }

            return null;
        };

        // subcomponents() can be used to get the subcomponents of a control
        let subcomponents = () => {
            return get_attr('SubComponents');
        }

        // Return an object that can be used to get information about these attributes
        return {
            all:            all,
            get_attr:       get_attr,
            subcomponents:  subcomponents
        };
    }

    ///
    /// Visits the flo items in the DOM, passing in attributes from
    /// the appropriate control data sections
    ///
    let visit_dom = (dom_node, control_data, visit_node) => {
        let attributes = get_attributes(control_data);

        // Visit the current node
        visit_node(dom_node, attributes);

        // Visit any subcomponents
        let subcomponents   = attributes.subcomponents();

        if (subcomponents !== null) {
            let subnodes    = [].slice.apply(dom_node.children).filter((node) => node.nodeType == Node.ELEMENT_NODE);

            for (let node_index=0; node_index<subcomponents.length; ++node_index) {
                visit_dom(subnodes[node_index], subcomponents[node_index], visit_node);
            }
        }
    }

    ///
    /// ===== HANDLING SERVER EVENTS
    ///

    ///
    /// Creates an event as part of a request
    ///
    let make_event = (kind, parameter) => {
        if (parameter === undefined) {
            return kind;
        } else {
            let res = {};
            res[kind] = parameter;
            return res;
        }
    }

    ///
    /// Creates a request for a particular session
    ///
    let make_request = (events, session_id) => {
        let res = { events: events };
        
        if (session_id) {
            res.session_id = session_id;
        }

        return res;
    }

    ///
    /// A new session has started
    ///
    let on_new_session = (new_session_id) => {
        return new Promise((resolve) => {
            note('Session ' + new_session_id);

            running_session_id = new_session_id;
            resolve();
        });
    }

    ///
    /// The entire UI HTML should be replaced with a new version
    ///
    let on_new_html = (new_user_interface_html, property_tree) => {
        return new Promise((resolve) => {
            let root = get_root();
            
            root.innerHTML = new_user_interface_html;

            visit_dom(root.children[0], property_tree, (node, attributes) => console.log(node, attributes.all(), attributes.subcomponents()));

            resolve();
        });
    }

    ///
    /// Dispatches updates in a request
    ///
    let dispatch_updates = (updates) => {
        // Each event generates a promise
        let update_promise  = Promise.resolve();
        let current_promise = update_promise;

        // We build the promise as we go
        updates.forEach((update) => {
            // serde encodes enums as objects, so we can tell what is what by looking at the first key
            let update_key = Object.keys(update)[0];

            switch (update_key) {
                case 'NewSession':
                    current_promise = current_promise.then(() => on_new_session(update[update_key]));
                    break;

                case 'NewUserInterfaceHtml':
                    current_promise = current_promise.then(() => on_new_html(update[update_key][0], update[update_key][1]));
                    break;

                default:
                    warn('Unknown update type', update_key, update);
                    break;
            }
        });

        return update_promise;
    }

    ///
    /// Sends a request to the session URI and processes the result
    ///
    let send_request = (request) => {
        return retry(() => http_post(request), () => warn('UI request failed - retrying'))
        .then((response) => response_to_object(response))
        .then((ui_request) => dispatch_updates(ui_request.updates))
        .catch((err) => {
            error('Could not refresh UI.', err);
        });
    }

    ///
    /// Makes a request to refresh the current state of the UI
    ///
    let refresh_ui = () => {
        let request = make_request([ make_event("UiRefresh") ], running_session_id);

        return send_request(request);
    }

    ///
    /// Makes the new session request
    ///
    let new_session = () => {
        let request = make_request([ make_event("NewSession") ]);

        // Generate a new session and immediately request that the UI be updated
        return send_request(request)
        .then(() => refresh_ui());
    }

    ///
    /// =====
    ///

    // All set up, let's go
    console.log('%c=== F L O W B E T W E E N ===', 'font-family: monospace; font-weight: bold; font-size: 150%;');
    new_session();
};

flowbetween();
