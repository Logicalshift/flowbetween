"use strict";

// FlowBetween
function flowbetween(root_node) {
    /// The ID of the running session
    let running_session_id = '';

    // Control data, starting at the root node
    let root_control_data = null;

    /// URL where the flowbetween session resides
    let target_url = '/flowbetween/session';

    let utf8 = new TextEncoder('utf-8');

    ///
    /// ===== INTERACTION
    /// 

    let commands = (function () {
        let command_list        = {};
        let commands_enabled    = false;

        ///
        /// Adds a new command with a particular name and description
        ///
        let add_command = (name, description, action) => {
            command_list[name] = {
                description:    description,
                action:         action
            }

            if (commands_enabled) {
                window[name] = action;
            }
        };

        ///
        /// Displays some help text
        ///
        let help = () => {
            console.log('');
            console.log('Functions available for FlowBetween');
            
            // Get the list of commands and find the length of the longest command
            let commands        = Object.keys(command_list).sort();
            let longest_command = commands.map((name) => name.length).reduce((a, b) => a>b ? a:b);

            for (let command_index=0; command_index < commands.length; ++command_index) {
                let command_name    = commands[command_index];
                let name_padding    = ' '.repeat(longest_command-command_name.length);
                let description     = command_list[command_name].description;

                console.log('  %c' + command_name + '()%c' + name_padding + ' - ' + description, 'font-weight: bold; font-family: monospace', 'font-weight: normal; font-family: monospace');
            }

            console.log('');
        };

        ///
        /// Enables any commands that might be defined
        ///
        let enable_commands = () => {
            // Copy the commands into the window object so they're available
            let commands = Object.keys(command_list);
            commands.forEach((command_name) => {
                window[command_name] = command_list[command_name].action;
            });

            commands_enabled = true;

            // Tell the user that the functions are available
            console.log('%cType %cflow_help()%c to see a list of functions for FlowBetween', 'font-family: monospace;', 'font-family: monospace; font-weight: bold', 'font-family: monospace; font-weight: normal;')
        };
        
        // The help command should always be available
        add_command('flow_help', 'Displays this message', help);

        return {
            add_command:        add_command,
            enable_commands:    enable_commands
        }
    })();

    let add_command     = commands.add_command;
    let enable_commands = commands.enable_commands;

    ///
    /// ===== LOGGING
    ///

    ///
    /// Note something
    ///
    let note = (function() {
        let recent_notes    = [];
        let show_notes      = false;

        let note = (msg) => {
            if (show_notes) {
                console.log('%c==> ' + msg, 'font-family: monospace; font-size: 80%; color: gray;');
            } else {
                recent_notes.push(msg);

                while (recent_notes.length > 100) {
                    recent_notes.shift();
                }
            }
        };

        add_command('show_notes', 'Displays verbose log messages', () => {
            if (!show_notes) {
                show_notes = true;

                recent_notes.forEach(msg => {
                    note(msg);
                });

                recent_notes    = [];

                note("Future notes will be displayed immediately");
            } else {
                note("Already showing notes");
            }
        });

        add_command('hide_notes', 'Hides verbose log messages', () => {
            if (show_notes) {
                note("Hiding future notes");
                show_notes = false;
            }
        });

        return note;
    })();

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
        return root_node;
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
        };

        // bounding_box retrieves the bounding box
        let bounding_box = () => {
            return get_attr('BoundingBox');
        };

        // controller retrieves the name of the controller for this subtree
        let controller = () => {
            return get_attr('Controller');
        };

        // actions returns the list of actions that apply to this control
        let actions = () => {
            let result = [];

            for (let attribute_index=0; attribute_index < attributes.length; ++attribute_index) {
                let attr        = attributes[attribute_index];
                let attr_name   = Object.keys(attr)[0];

                if (attr_name === 'Action') {
                    result.push(attr[attr_name]);
                }
            }

            return result.length>0 ? result : null;
        }

        // Return an object that can be used to get information about these attributes
        return {
            all:            all,
            get_attr:       get_attr,
            subcomponents:  subcomponents,
            controller:     controller,
            actions:        actions,
            bounding_box:   bounding_box
        };
    }

    ///
    /// Visits the flo items in the DOM, passing in attributes from
    /// the appropriate control data sections
    ///
    let visit_dom = (dom_node, control_data, visit_node) => {
        // visit_internal tracks the controller path for each node
        let visit_internal = (dom_node, control_data, visit_node, controller_path) => {
            let attributes = get_attributes(control_data);
            
            // Visit the current node
            visit_node(dom_node, attributes, controller_path);

            // If this node has a controller, it's applied as part of the path for the child nodes
            let child_node_path = controller_path;
            let controller      = attributes.controller();

            if (controller) {
                child_node_path = child_node_path.slice();
                child_node_path.push(controller);
            }
    
            // Visit any subcomponents
            let subcomponents   = attributes.subcomponents();
    
            if (subcomponents !== null) {
                let subnodes    = [].slice.apply(dom_node.children).filter((node) => node.nodeType === Node.ELEMENT_NODE);
    
                for (let node_index=0; node_index<subcomponents.length; ++node_index) {
                    visit_internal(subnodes[node_index], subcomponents[node_index], visit_node, child_node_path);
                }
            }
        };

        // Initial node has no controller path
        visit_internal(dom_node, control_data, visit_node, []);
    }

    ///
    /// Computes a position, given a previous position and a position element
    ///
    let layout_position = (next_pos_desc, last_pos_abs, max_extent, total_stretch, stretch_extent) => {
        let pos_type;
        
        if (typeof(next_pos_desc) === 'string') {
            pos_type = next_pos_desc;
        } else {
            pos_type = Object.keys(next_pos_desc)[0];            
        }

        switch (pos_type) {
            case 'At':      return next_pos_desc[pos_type];
            case 'Offset':  return last_pos_abs + next_pos_desc[pos_type];
            case 'Start':   return 0;
            case 'End':     return max_extent;
            case 'After':   return last_pos_abs;

            case 'Stretch': {
                let stretch = next_pos_desc[pos_type];
                if (total_stretch > 0) {
                    let ratio = stretch/total_stretch;
                    return last_pos_abs + stretch_extent*ratio;
                } else {
                    return last_pos_abs;
                }
            }
            
            default:
                warn('Unknown position type', next_pos_desc);
                return last_pos_abs;
        }
    }

    ///
    /// Lays out the subcomponents associated with a particular node
    ///
    let layout_subcomponents = (parent_node, attributes) => {
        let subcomponents   = attributes.subcomponents();
        let subnodes        = parent_node.children;
        let positions       = [];
        let total_width     = parent_node.clientWidth;
        let total_height    = parent_node.clientHeight;

        if (subcomponents === null) {
            return;
        }

        // First pass: position all of the nodes, assuming stretch nodes have 0 width/height
        let xpos        = 0;
        let ypos        = 0;
        let stretch_x   = 0;
        let stretch_y   = 0;

        let default_bounding_box = {
            x1: 'Start',
            x2: 'Start',
            y1: 'End',
            y2: 'End'
        };

        for (let node_index=0; node_index<subcomponents.length; ++node_index) {
            let component       = subcomponents[node_index];
            let bounding_box    = get_attributes(component).bounding_box() || default_bounding_box;

            // Convert the bounding box into an absolute position
            let abs_pos         = {
                x1: layout_position(bounding_box.x1, xpos, total_width),
                y1: layout_position(bounding_box.y1, ypos, total_height),
                x2: layout_position(bounding_box.x2, xpos, total_width),
                y2: layout_position(bounding_box.y2, ypos, total_height)
            };

            positions.push(abs_pos);

            // The x2, y2 coordinate forms the coord for the next part
            xpos = abs_pos.x2;
            ypos = abs_pos.y2;

            // Update the total amount of 'stretch' ratio across the whole collection
            stretch_x += bounding_box.x1['Stretch'] || 0;
            stretch_x += bounding_box.x2['Stretch'] || 0;
            stretch_y += bounding_box.y1['Stretch'] || 0;
            stretch_y += bounding_box.y2['Stretch'] || 0;
        }

        // Second pass: lay out stretch nodes
        if (stretch_x > 0 || stretch_y > 0) {
            // Work out the amount of space we have to stretch into
            let stretch_width   = total_width - xpos;
            let stretch_height  = total_height - ypos;

            // Clear the positions
            positions = [];
            xpos = 0;
            ypos = 0;

            // Relayout
            for (let node_index=0; node_index<subcomponents.length; ++node_index) {
                let component       = subcomponents[node_index];
                let bounding_box    = get_attributes(component).bounding_box() || default_bounding_box;
    
                // Convert the bounding box into an absolute position
                let abs_pos         = {
                    x1: layout_position(bounding_box.x1, xpos, total_width, stretch_x, stretch_width),
                    y1: layout_position(bounding_box.y1, ypos, total_height, stretch_y, stretch_height),
                    x2: layout_position(bounding_box.x2, xpos, total_width, stretch_x, stretch_width),
                    y2: layout_position(bounding_box.y2, ypos, total_height, stretch_y, stretch_height)
                };
    
                positions.push(abs_pos);
    
                // The x2, y2 coordinate forms the coord for the next part
                xpos = abs_pos.x2;
                ypos = abs_pos.y2;
            }
        }

        // Final pass: finish the layout
        for (let node_index=0; node_index<subcomponents.length; ++node_index) {
            let element = subnodes[node_index];
            let pos     = positions[node_index];

            element.style.left      = pos.x1 + 'px';
            element.style.width     = (pos.x2-pos.x1) + 'px';
            element.style.top       = pos.y1 + 'px';
            element.style.height    = (pos.y2-pos.y1) + 'px';
        }
    };

    ///
    /// Wires up a click action to a node
    ///
    let wire_click = (action_action, node, controller_path) => {
        node.addEventListener("click", () => {
            note("Click " + action_action + " --> " + controller_path);
        });
    }

    ///
    /// Wires up an action to a node
    ///
    let wire_action = (action, node, controller_path) => {
        let action_type     = action[0];
        let action_action   = action[1];

        switch (action_type) {
            case 'Click':
                wire_click(action_action, node, controller_path);
                break;

            default:
                warn('Unknown action type: ' + action_type);
                break;
        }
    };

    ///
    /// Wires up events for a component
    ///
    let wire_events = (node, attributes, controller_path) => {
        let actions = attributes.actions();

        if (actions) {
            actions.forEach(action => wire_action(action, node, controller_path));
        }
    };

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
    /// Given a node and its control data, updates the layout
    ///
    let layout_tree = (dom_node, control_data) => {
        visit_dom(dom_node, control_data, (node, attributes) => layout_subcomponents(node, attributes));
    }

    ///
    /// Given a node and its control data, wires up any events
    ///
    /// TODO: this currently only tracks the controller path from the root so won't work when updating the tree
    ///
    let wire_tree = (dom_node, control_data) => {
        visit_dom(dom_node, control_data, (node, attributes, controller_path) => wire_events(node, attributes, controller_path));
    }

    ///
    /// The entire UI HTML should be replaced with a new version
    ///
    let on_new_html = (new_user_interface_html, property_tree) => {
        return new Promise((resolve) => {
            let root = get_root();
            
            // Update the DOM
            root.innerHTML      = new_user_interface_html;
            root_control_data   = property_tree;

            // Perform initial layout
            wire_tree(root.children[0], root_control_data);
            layout_tree(root.children[0], root_control_data);

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

    // Whenever the document resizes, lay everything out again
    let willResize = false;
    window.addEventListener('resize', () => {
        if (!willResize) {
            willResize = true;

            requestAnimationFrame(() => {
                willResize = false;

                if (root_control_data) {
                    layout_tree(get_root().children[0], root_control_data);
                }
            });
        }
    });

    // All set up, let's go
    console.log('%c=== F L O W B E T W E E N ===', 'font-family: monospace; font-weight: bold; font-size: 150%;');
    new_session();
    enable_commands();
};

document.addEventListener("DOMContentLoaded", () => flowbetween(document.getElementById('root')));
