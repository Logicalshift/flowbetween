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
    /// Functions related to templating
    ///
    let templating = (function() {
        /// Template DOM nodes, ready to be applied
        let templates = {};

        ///
        /// Loads the UI templates for a particular DOM node
        ///
        let reload_templates = (root_node) => {
            // Clear the DOM nodes
            templates = {};

            // Find the template elements beneath the root node
            let rootTemplates = root_node.getElementsByTagName('TEMPLATE');

            // Each template can define the nodes we apply to flo-nodes, by example
            for (let templateNumber=0; templateNumber<rootTemplates.length; ++templateNumber) {
                let templateParent = rootTemplates[templateNumber].content;
                
                for (let nodeIndex=0; nodeIndex<templateParent.children.length; ++nodeIndex) {
                    let templateNode = templateParent.children[nodeIndex];
                    let templateName = templateNode.tagName.toLowerCase();

                    templates[templateName] = [].slice.apply(templateNode.children);
                }
            }
        };

        ///
        /// Applies a template to a node if possible
        ///
        /// Note that if we've wired up events, we won't re-wire them
        /// as part of this call, so that's something that needs to be
        /// done.
        ///
        let apply_template = (node, attributes) => {
            // Get the template elements for this node
            let templateForNode = templates[node.tagName.toLowerCase()];

            if (templateForNode) {
                // Remove any existing template nodes
                get_decorative_subnodes(node).forEach(decoration => node.removeChild(decoration));

                // Copy each template element
                let newNodes = templateForNode.map(templateNode => document.importNode(templateNode, true));

                // Add the nodes to this node
                let firstNode = node.children.length > 0 ? node.children[0] : null;

                newNodes.forEach(newNode => node.insertBefore(newNode, firstNode));
            }
        };

        add_command('show_templates', 'Displays the template nodes', () => console.log(templates));

        return {
            reload_templates: reload_templates,
            apply_template: apply_template
        };
    })();

    let reload_templates    = templating.reload_templates;
    let apply_template      = templating.apply_template;

    ///
    /// Fetches the root of the UI
    ///
    let get_root = () => {
        return root_node;
    }

    ///
    /// Give a DOM node, returns the child nodes that represent flowbetween controls
    ///
    let get_flo_subnodes = (node) => {
        return [].slice.apply(node.children).filter(element => element.nodeType === Node.ELEMENT_NODE && element.tagName.toLowerCase().startsWith("flo-"));
    }

    ///
    /// Given a DOM node, returns the child nodes that represent decorative items
    ///
    let get_decorative_subnodes = (node) => {
        return [].slice.apply(node.children).filter(element => element.nodeType === Node.ELEMENT_NODE && !element.tagName.toLowerCase().startsWith("flo-"));
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
        let get_attrs = (name) => {
            let result = [];
            
            for (let attribute_index=0; attribute_index < attributes.length; ++attribute_index) {
                let attr        = attributes[attribute_index];
                let attr_name   = Object.keys(attr)[0];

                if (attr_name === name) {
                    result.push(attr[attr_name]);
                }
            }

            return result.length>0 ? result : null;
        };

        // get_attr(name) will retrieve the attribute with the given name (or null if it does not exist)
        let get_attr = (name) => {
            let result = get_attrs(name);
            return result ? result[0] : null;
        }

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
            return get_attrs('Action');
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
                let subnodes = get_flo_subnodes(dom_node);
    
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
        let subnodes        = get_flo_subnodes(parent_node);
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
    /// Adds an action event to a flo node
    ///
    let add_action_event = (node, event_name, handler, options) => {
        // We add action events to the node and any decorations it may have
        let event_nodes = [node];
        [].push.apply(event_nodes, get_decorative_subnodes(node));

        // Add the event
        event_nodes.forEach(node => node.addEventListener(event_name, handler, options));

        // Update the function that removes events from this node
        let remove_more_events = node.flo_remove_actions;
        node.flo_remove_actions = () => {
            event_nodes.forEach(node => node.removeEventListener(event_name, handler));
            event_nodes = [];

            if (remove_more_events) {
                remove_more_events();
            }
        }
    }

    ///
    /// Clears any events attached to a DOM node
    ///
    let remove_action_events_from_node = (node) => {
        // The flo_remove_actions property attached to a DOM node is used to get rid of any events we might have attached to it
        let remove_events       = node.flo_remove_actions;
        node.flo_remove_actions = null;
        if (remove_events) {
            remove_events();
        }
    }

    ///
    /// Wires up a click action to a node
    ///
    let wire_click = (action_name, node, controller_path) => {
        add_action_event(node, "click", () => {
            note("Click " + action_name + " --> " + controller_path);

            perform_action(controller_path, action_name);
        });
    }

    ///
    /// Wires up an action to a node
    ///
    let wire_action = (action, node, controller_path) => {
        // If this node is already wired up, remove the events we added
        remove_action_events_from_node(node);

        // Store the actions for this event
        let action_type = action[0];
        let action_name = action[1];

        switch (action_type) {
            case 'Click':
                wire_click(action_name, node, controller_path);
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
        // Remove existing events, if any
        if (node.flo_remove_actions) {
            remove_action_events_from_node(node);
        }

        // Fetch actions
        let actions = attributes.actions();

        if (actions) {
            actions.forEach(action => wire_action(action, node, controller_path));
        }
    };

    ///
    /// ===== VIEWMODEL
    ///

    let viewmodel = (function() {
        let viewmodel = {
            subcontrollers: {},
            keys:           {}
        };

        ///
        /// Returns the viewmodel for a particular controller
        ///
        let viewmodel_for_controller = (controller_path) => {
            let viewmodel_for_controller = (controller_path, viewmodel) => {
                // It's just the current viewmodel if there's no path remaining to folow
                if (controller_path.length === 0) {
                    return viewmodel;
                }
                
                // Follow the path
                let next_controller = controller_path[0];
                let next_viewmodel  = viewmodel.subcontrollers[next_controller];

                // We always ensure that there's a viewmodel for any requested path (so we create a new viewmodel as a side-effect)
                if (!next_viewmodel) {
                    next_viewmodel = {
                        subcontrollers: {},
                        keys:           {}
                    }

                    viewmodel.subcontrollers[next_controller] = next_viewmodel;
                }

                // Recursively follow the path to get the viewmodel for this controller
                let remaining_path = controller_path.slice(1);
                return viewmodel_for_controller(remaining_path, next_viewmodel);
            }

            return viewmodel_for_controller(controller_path, viewmodel);
        }

        ///
        /// Sets a single value in a controller
        ///
        let set_viewmodel_value = (controller_path, key, value) => {
            let viewmodel = viewmodel_for_controller(controller_path);
            viewmodel.keys[key] = value;
        }

        ///
        /// Retrieves a viewmodel value for a particular controller
        ///
        let get_viewmodel_value = (controller_path, key) => {
            let viewmodel = viewmodel_for_controller(controller_path);
            return viewmodel.keys[key] || "Nothing";
        }

        ///
        /// Given a controller, replaces its entire view model
        ///
        let set_viewmodel = (controller_path, new_viewmodel_keys) => {
            let viewmodel   = viewmodel_for_controller(controller_path);
            viewmodel.keys  = new_viewmodel_keys;
        }

        return {
            set_viewmodel_value:    set_viewmodel_value,
            get_viewmodel_value:    get_viewmodel_value,
            set_viewmodel:          set_viewmodel
        }
    })();

    /// Sets a new view model for a controller
    let set_viewmodel_value = viewmodel.set_viewmodel_value;
    let get_viewmodel_value = viewmodel.get_viewmodel_value;
    let set_viewmodel       = viewmodel.set_viewmodel;

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
        visit_dom(dom_node, control_data, (node, attributes, controller_path) => {
            // Store the attributes for this node for convenience
            node.flo = {
                controller: controller_path,
                attributes: attributes
            };

            // Attach any events that this node might require
            wire_events(node, attributes, controller_path);
        });
    }

    ///
    /// Applies the node templates to a DOM tree
    ///
    let apply_templates_to_tree = (dom_node, control_data) => {
        visit_dom(dom_node, control_data, (node, attributes) => apply_template(node, attributes));
    }

    ///
    /// The entire UI HTML should be replaced with a new version
    ///
    let on_new_html = (new_user_interface_html, property_tree) => {
        note('Updating user interface');

        return new Promise((resolve) => {
            let root = get_root();
            
            // Update the DOM
            root.innerHTML      = new_user_interface_html;
            root_control_data   = property_tree;

            // Perform initial layout
            apply_templates_to_tree(get_flo_subnodes(root)[0], root_control_data);
            wire_tree(get_flo_subnodes(root)[0], root_control_data);
            layout_tree(get_flo_subnodes(root)[0], root_control_data);

            resolve();
        });
    }

    ///
    /// Dispatches updates in a request
    ///
    let dispatch_updates = (function() {
        let show_updates = false;
        add_command('show_update_events', 'Log the update objects from the server', () => show_updates = true);
        add_command('hide_update_events', 'Stop logging the update objects from the server', () => show_updates = false);
        
        return (updates) => {
            if (show_updates) {
                console.log('Dispatching updates', updates);
            }

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
        };
    })();

    ///
    /// Sends a request to the session URI and processes the result
    ///
    let send_request = (request) => {
        return retry(() => http_post(request), () => warn('UI request failed - retrying'))
        .then((response) => response_to_object(response))
        .then((ui_request) => dispatch_updates(ui_request.updates))
        .catch((err) => {
            error('Request failed.', err);
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
    /// Performs a particular action
    ///
    let perform_action = (controller_path, action_name) => {
        let request = make_request([ make_event({ Action: [controller_path, action_name] })], running_session_id);

        return send_request(request);
    }

    ///
    /// ===== STARTUP
    ///

    // Whenever the document resizes, lay everything out again
    let willResize = false;
    window.addEventListener('resize', () => {
        if (!willResize) {
            willResize = true;

            requestAnimationFrame(() => {
                willResize = false;

                if (root_control_data) {
                    layout_tree(get_flo_subnodes(get_root())[0], root_control_data);
                }
            });
        }
    });

    // All set up, let's go
    console.log('%c=== F L O W B E T W E E N ===', 'font-family: monospace; font-weight: bold; font-size: 150%;');
    reload_templates(document.getRootNode());
    new_session();
    enable_commands();
};

///
/// Declares a function that propagates events from the document of an
/// object node to the parent of the object node
///
/// This gets around the problem that the 'internal' document in an 
/// object does not bubble events to the containing elements
///
let propagate_object_events = (function() {
    let default_events = [
        "blur",
        "click",
        "dblclick",
        "input",
        "mousedown",
        "mousemove",
        "mouseout",
        "mouseover",
        "mouseup",
        "pointercancel", 
        "pointerdown", 
        "pointerenter", 
        "pointerleave",
        "pointermove",
        "pointerout",
        "pointerover",
        "pointerup",
        "touchcancel",
        "touchmove",
        "touchstart",
        "wheel"
    ];

    return (object_node, events) => {
        events = events || default_events;

        let document        = object_node.contentDocument;
        let root_element    = document.children[0];

        events.forEach(event => {
            root_element.addEventListener(event, e => {
                // Can't dispatch the event while it's already being dispatched so we wait a frame
                requestAnimationFrame(() => object_node.parentNode.dispatchEvent(e));
            });
        });
    };
})();
