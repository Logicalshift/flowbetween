'use strict';

//  ___ _            ___      _                       
// | __| |_____ __ _| _ ) ___| |___ __ _____ ___ _ _  
// | _|| / _ \ V  V / _ \/ -_)  _\ V  V / -_) -_) ' \ 
// |_| |_\___/\_/\_/|___/\___|\__|\_/\_/\___\___|_||_|
//                                                

/* exported flowbetween */
/* exported replace_object_with_content */
/* global flo_canvas, flo_paint */

function flowbetween(root_node) {
    /// The ID of the running session
    let running_session_id = '';

    // Control data, starting at the root node
    let root_control_data = null;

    // URL where the flowbetween session resides
    let target_url = '/flowbetween/session';

    // UTF encoder
    let utf8 = new TextEncoder('utf-8');

    // Find out where we're running
    let doc_url  = document.createElement('a');
    doc_url.href = document.URL;
    let base_url = doc_url.protocol + '//' + doc_url.host;
    
    // Some utility functions
    Array.prototype.mapMany = function (map_fn) {
        let self = this;
        return Array.prototype.concat.apply([], self.map(map_fn));
    };

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
            };

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
            console.log('%cType %cflow_help()%c to see a list of functions for FlowBetween', 'font-family: monospace;', 'font-family: monospace; font-weight: bold', 'font-family: monospace; font-weight: normal;');
        };
        
        // The help command should always be available
        add_command('flow_help', 'Displays this message', help);

        return {
            add_command:        add_command,
            enable_commands:    enable_commands
        };
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

                note('Future notes will be displayed immediately');
            } else {
                note('Already showing notes');
            }
        });

        add_command('hide_notes', 'Hides verbose log messages', () => {
            if (show_notes) {
                note('Hiding future notes');
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
    };
    
    ///
    /// Display an error
    ///
    let error = function() {
        console.error.apply(console, arguments);
    };

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
    };

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
            if (method !== 'GET') {
                req.send(utf8.encode(encoding));
            } else {
                req.send();
            }
        });
    };

    /// Sends a POST request
    let http_post   = (obj, url) => xhr(obj, url, 'POST');

    /// Sets a GET request
    let http_get    = (url) => xhr({}, url, 'GET');

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
    };

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
            };
            
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
        let apply_template = (node) => {
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

        ///
        /// Finds all of the <object> nodes in templates underneath a root node and loads their content.
        /// If they are inlinable (eg, they are SVG files, which is the expected case), then inline them.
        ///
        /// SVG files in particular can be in objects but have more useful properties outside of them
        /// (eg, as they can be affected by CSS settings on their container this way). However, they are 
        /// ugly to inline in HTML if they are of any complexity, so it's nice to be able to reference them 
        /// externally. Loading them every time when a template is re-used is nefficient too, so this 
        /// provides a slightly nicer way to deal with SVG UI elements.
        ///
        let inline_template_objects = (root_node) => {
            return new Promise((resolve, reject) => {
                // Find all of the objects in the document
                let templates   = [].slice.apply(root_node.getElementsByTagName('TEMPLATE'));
                let objects     = templates
                    .map(template => template.content.children[0])
                    .mapMany(template => [].slice.apply(template.getElementsByTagName('OBJECT')));

                // Retrieves an absolute URL from a relative one for our document
                let get_absolute_url = (relative_url) => {
                    let a = document.createElement('a');
                    a.href = relative_url;
                    return a.href;
                };

                // Performs inlining of a SVG
                let inline_svg = (obj_node, svg) => {
                    // Generate a node from the SVG
                    let fake_root = document.createElement('div');
                    fake_root.innerHTML = svg;
                    let svg_node = fake_root.children[0];

                    // Splice in place of the obj node
                    let parent = obj_node.parentNode;

                    parent.insertBefore(svg_node, obj_node.nextSibling);
                    parent.removeChild(obj_node);
                };

                // Try to load all of the objects
                let load_objects    = objects.map(obj_node => {
                    let object_url = get_absolute_url(obj_node.getAttribute('data'));

                    return http_get(object_url).then(object_request => {
                        let content_type = object_request.getResponseHeader('Content-Type');

                        if (content_type.includes('image/svg+xml')) {
                            let svg_content = object_request.response;
                            inline_svg(obj_node, svg_content);
                        }
                    });
                });

                // Promise is done once all of the objects are loaded
                Promise.all(load_objects)
                    .then(() => resolve())
                    .catch(ex => reject(ex));
            });
        };

        add_command('show_templates', 'Displays the template nodes', () => console.log(templates));

        return {
            reload_templates:           reload_templates,
            apply_template:             apply_template,
            inline_template_objects:    inline_template_objects
        };
    })();

    let reload_templates        = templating.reload_templates;
    let apply_template          = templating.apply_template;
    let inline_template_objects = templating.inline_template_objects;

    ///
    /// Fetches the root of the UI
    ///
    let get_root = () => {
        return root_node;
    };

    ///
    /// Give a DOM node, returns the child nodes that represent flowbetween controls
    ///
    let get_flo_subnodes = (node) => {
        return [].slice.apply(node.children).filter(element => element.nodeType === Node.ELEMENT_NODE && element.tagName.toLowerCase().startsWith('flo-'));
    };

    ///
    /// Given a DOM node, returns the child nodes that represent decorative items
    ///
    let get_decorative_subnodes = (node) => {
        return [].slice.apply(node.children).filter(element => element.nodeType === Node.ELEMENT_NODE && !element.tagName.toLowerCase().startsWith('flo-'));
    };

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
            return get_attrs('Action');
        };

        // Return an object that can be used to get information about these attributes
        return {
            all:            all,
            get_attr:       get_attr,
            subcomponents:  subcomponents,
            controller:     controller,
            actions:        actions,
            bounding_box:   bounding_box
        };
    };

    ///
    /// Adds a class to the className of a DOM node
    ///
    let add_class = (dom_node, class_name) => {
        let original_class_name     = dom_node.className;
        let class_name_components   = original_class_name.split(' ');
        let new_components          = class_name_components.filter(name => name !== class_name);

        new_components.push(class_name);

        dom_node.className = new_components.join(' ');
    };

    ///
    /// Removes a class to the className of a DOM node
    ///
    let remove_class = (dom_node, class_name) => {
        let original_class_name     = dom_node.className;
        let class_name_components   = original_class_name.split(' ');
        let new_components          = class_name_components.filter(name => name !== class_name);

        dom_node.className = new_components.join(' ');
    };

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
    };

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
    };

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
        // addEventListener can only add a single handler for a particular event, but we want to be able to support multiple
        let event_property  = 'flo_event_' + event_name;
        let current_event   = node[event_property];

        if (current_event) {
            let original_handler = handler;
            handler = event => {
                current_event(event);
                original_handler(event);
            };
        }

        // We add action events to the node and any decorations it may have
        let event_nodes = [node];
        [].push.apply(event_nodes, get_decorative_subnodes(node));

        // Add the event
        if (current_event) {
            event_nodes.forEach(node => node.removeEventListener(event_name, current_event));
        }
        event_nodes.forEach(node => node.addEventListener(event_name, handler, options));
        node[event_property] = handler;

        // Update the function that removes events from this node
        let remove_more_events = node.flo_remove_actions;
        node.flo_remove_actions = () => {
            event_nodes.forEach(node => node.removeEventListener(event_name, handler));
            event_nodes = [];

            if (remove_more_events) {
                remove_more_events();
            }
        };
    };

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
    };

    ///
    /// Wires up a click action to a node
    ///
    let wire_click = (action_name, node, controller_path) => {
        add_action_event(node, 'click', event => {
            event.preventDefault();
            note('Click ' + action_name + ' --> ' + controller_path);

            perform_action(controller_path, action_name, null);
        });
    };

    ///
    /// Wires up an action to a node
    ///
    let wire_action = (action, node, controller_path) => {
        // If this node is already wired up, remove the events we added
        remove_action_events_from_node(node);

        // Store the actions for this event
        let action_type = action[0];
        let action_name = action[1];

        if (action_type === 'Click') {
            wire_click(action_name, node, controller_path);

        } else if (action_type['Paint']) {
            flo_paint.wire_paint(action_type['Paint'], action_name, node, controller_path);

        } else {
            warn('Unknown action type: ' + action_type);
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
    /// Binds a single attribute to a node
    ///
    let bind_viewmodel_attribute = (node, attribute, controller_path) => {
        let remove_action = null;

        if (attribute['Selected']) {
            // The selected property updates the node class
            remove_action = on_property_change(controller_path, attribute['Selected'], is_selected => {
                if (is_selected['Bool']) {
                    add_class(node, 'selected');
                } else {
                    remove_class(node, 'selected');
                }

                return true;
            });
        }

        // Update the property that allows us to unbind the viewmodel
        if (remove_action) {
            let previous_unbind = node.flo_unbind_viewmodel;
            node.flo_unbind_viewmodel = () => {
                remove_action();
                if (previous_unbind) {
                    previous_unbind();
                }
            };
        }
    };

    ///
    /// Binds any viemwodel attributes for a node
    ///
    let bind_viewmodel = (node, attributes, controller_path) => {
        // Ensure that any previous viewmodel attached to this node is removed
        let unbind_viewmodel = node.flo_unbind_viewmodel || (() => {});
        unbind_viewmodel();
        
        // Bind the attributes to this node
        attributes.all().forEach(attribute => bind_viewmodel_attribute(node, attribute, controller_path));
    };

    ///
    /// ===== VIEWMODEL
    ///

    let viewmodel = (function() {
        let viewmodel = {
            subcontrollers: {},
            keys:           {},
            actions:        {}
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
                        keys:           {},
                        actions:        {}
                    };

                    viewmodel.subcontrollers[next_controller] = next_viewmodel;
                }

                // Recursively follow the path to get the viewmodel for this controller
                let remaining_path = controller_path.slice(1);
                return viewmodel_for_controller(remaining_path, next_viewmodel);
            };

            return viewmodel_for_controller(controller_path, viewmodel);
        };

        ///
        /// Processes a viewmodel update event
        ///
        let process_viewmodel_update = (update_data) => {
            let controller_path = update_data.controller_path;
            let updates         = update_data.updates;
            
            // Process the updates for this controller
            let viewmodel = viewmodel_for_controller(controller_path);
            if (!viewmodel.keys) {
                viewmodel.keys = {};
            }

            updates.forEach(update => viewmodel.keys[update[0]] = update[1]);

            // Notify anything that's listening of the changes
            updates.forEach(update => {
                let key         = update[0];
                let new_value   = update[1];
                let actions     = viewmodel.actions[key] || [];

                actions.forEach(action => {
                    if (action) {
                        action(new_value);
                    }
                });
            });
        };

        ///
        /// Performs an action when a viewmodel value changes. Returns a function
        /// that will disable this action.
        ///
        /// The event will be invoked immediately with the current value of the
        /// key, if it has one. The event function should return true if it
        /// wishes to process future events.
        ///
        let on_viewmodel_change = (controller_path, key, change_action) => {
            // Get the actions for the viewmodel for this controller
            let viewmodel   = viewmodel_for_controller(controller_path);
            let actions     = viewmodel.actions;

            // Change action should disable itself if it returns false
            let action_with_remove = (key_value) => {
                if (change_action !== null) {
                    let should_remove = !change_action(key_value);
                    if (should_remove) {
                        remove_action();
                    }
                } else {
                    remove_action();
                }
            };

            // Create or retrieve the list of actions for this path
            let actions_for_key = actions[key];
            if (!actions_for_key) {
                actions_for_key = actions[key] = [];                
            }

            // Add in the change action
            // It replaces a 'null' entry left by a previous action or is added to the end
            let action_index = actions_for_key.findIndex(item => item === null);
            if (action_index === -1) {
                action_index = actions_for_key.length;
                actions_for_key.push(null);
            }

            actions_for_key[action_index] = action_with_remove;

            // Create the removal function
            let removed         = false;
            let remove_action   = () => {
                if (!removed) {
                    removed = true;
                    actions_for_key[action_index] = null;
                } else {
                    note('Double removal of action');
                }
            };

            // Fire the event immediately if the key has a value
            let key_value = viewmodel.keys[key];
            if (key_value) {
                action_with_remove(key_value);
            }

            return remove_action;
        };

        ///
        /// Calls an event when a property changes, returning a function that will
        /// unbind the event and calling the action at least once on initialisation.
        ///
        let on_property_change = (controller_path, property, change_action) => {
            if (property.hasOwnProperty('Bind')) {
                return on_viewmodel_change(controller_path, property['Bind'], change_action);
            } else {
                change_action(property);
                return () => {};
            }
        };

        ///
        /// Convenience command to dump out the viewmodel
        ///
        add_command('show_viewmodel', 'Writes the viewmodel to the console', () => {
            let display_controller = (controller_name, viewmodel) => {
                console.group(controller_name);
                console.log(viewmodel.keys);

                Object.keys(viewmodel.subcontrollers)
                    .forEach(subcontroller => display_controller(subcontroller, viewmodel.subcontrollers[subcontroller]));
                console.groupEnd();
            };

            display_controller('Flowbetween', viewmodel);
        });

        return {
            process_viewmodel_update:   process_viewmodel_update,
            on_property_change:         on_property_change
        };
    })();

    let process_viewmodel_update    = viewmodel.process_viewmodel_update;
    let on_property_change          = viewmodel.on_property_change;

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
    };

    ///
    /// Creates a request for a particular session
    ///
    let make_request = (events, session_id) => {
        let res = { events: events };
        
        if (session_id) {
            res.session_id = session_id;
        }

        return res;
    };

    ///
    /// A new session has started
    ///
    let on_new_session = (new_session_id) => {
        return new Promise((resolve) => {
            note('Session ' + new_session_id);

            running_session_id = new_session_id;
            resolve();
        });
    };

    ///
    /// Given a node and its control data, updates the layout
    ///
    let layout_tree = (dom_node, control_data) => {
        visit_dom(dom_node, control_data, (node, attributes) => layout_subcomponents(node, attributes));
    };

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
    };

    ///
    /// Applies the node templates to a DOM tree
    ///
    let apply_templates_to_tree = (dom_node, control_data) => {
        visit_dom(dom_node, control_data, (node, attributes) => apply_template(node, attributes));
        visit_dom(dom_node, control_data, (node) => { if (node.tagName.toLowerCase() === 'flo-canvas') flo_canvas.start(node); });
    };

    ///
    /// Binds the viewmodel to a DOM tree
    ///
    let bind_viewmodel_to_tree = (dom_node, control_data) => {
        visit_dom(dom_node, control_data, (node, attributes, controller_path) => bind_viewmodel(node, attributes, controller_path));
    };

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
            bind_viewmodel_to_tree(get_flo_subnodes(root)[0], root_control_data);
            wire_tree(get_flo_subnodes(root)[0], root_control_data);
            layout_tree(get_flo_subnodes(root)[0], root_control_data);

            resolve();
        });
    };

    ///
    /// The entire viewmodel should be replaced with a new version
    ///
    let on_new_viewmodel = (viewmodel_update_list) => {
        note('Replacing viewmodel');
        
        return new Promise((resolve) => {
            viewmodel_update_list.forEach(update => process_viewmodel_update(update));
            resolve();
        });
    };

    ///
    /// Handles a viewmodel update event
    ///
    let on_update_viewmodel = (viewmodel_update_list) => {
        note('Updating viewmodel');

        return new Promise((resolve) => {
            viewmodel_update_list.forEach(update => process_viewmodel_update(update));
            resolve();
        });
    };

    ///
    /// Handles a canvas update event
    ///
    let on_update_canvas = (canvas_update_list) => {
        note('Updating canvases');

        return new Promise((resolve) => {
            canvas_update_list.forEach(update => {
                let controller  = update['controller'];
                let canvas_name = update['canvas_name'];
                let updates     = update['updates'];

                flo_canvas.update_canvas(controller, canvas_name, updates);
            });

            resolve();
        });
    };

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
                if (update['NewSession']) {

                    current_promise = current_promise
                        .then(() => on_new_session(update['NewSession']));

                } else if (update['NewUserInterfaceHtml']) {

                    let new_ui_html     = update['NewUserInterfaceHtml'];
                    let html            = new_ui_html[0];
                    let property_tree   = new_ui_html[1];
                    let viewmodel       = new_ui_html[2];

                    current_promise = current_promise
                        .then(() => on_new_viewmodel(viewmodel))
                        .then(() => on_new_html(html, property_tree));

                } else if (update['UpdateViewModel']) {

                    let updates = update['UpdateViewModel'];

                    current_promise = current_promise
                        .then(() => on_update_viewmodel(updates));
                
                } else if (update['UpdateCanvas']) {

                    let updates = update['UpdateCanvas'];

                    current_promise = current_promise
                        .then(() => on_update_canvas(updates));

                } else {
                    warn('Unknown update type', Object.keys(update)[0], update);
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
    };

    ///
    /// Makes a request to refresh the current state of the UI
    ///
    let refresh_ui = () => {
        let request = make_request([ make_event('UiRefresh') ], running_session_id);

        return send_request(request);
    };

    ///
    /// Makes the new session request
    ///
    let new_session = () => {
        let request = make_request([ make_event('NewSession') ]);

        // Generate a new session and immediately request that the UI be updated
        return send_request(request)
            .then(() => refresh_ui());
    };

    ///
    /// Performs a particular action
    ///
    let perform_action = (controller_path, action_name, action_parameter) => {
        let request = make_request([ make_event({ Action: [controller_path, action_name, action_parameter || 'None'] })], running_session_id);

        return send_request(request);
    };

    ///
    /// ====== DEBUGGING AND INTROSPECTION
    ///

    add_command('canvas_stats', 'Display statistics about the canvases in this window', () => {
        let canvases = [].slice.apply(document.getElementsByTagName('flo-canvas'));

        canvases.forEach(canvas => {
            if (canvas.flo_draw) {
                console.log(canvas);
                console.log(canvas.flo_draw.stats());
            }
        });
    });

    add_command('canvas_replay', 'Replays all of the canvases and reports timings', () => {
        let canvases = [].slice.apply(document.getElementsByTagName('flo-canvas'));

        canvases.forEach(canvas => {
            if (canvas.flo_draw) {
                let start_time = Date.now();
                for (let iter=0; iter<10; ++iter) {
                    canvas.flo_draw.replay_drawing();
                }
                let total_time = Date.now() - start_time;

                console.log('Redraw: ' + canvas.flo_controller + '/' + canvas.flo_name + ': ' + total_time/10 + 'ms');

                start_time = Date.now();
                for (let iter=0; iter<10; ++iter) {
                    canvas.flo_draw.draw_layers();
                }
                total_time = Date.now() - start_time;

                console.log('Draw layers: ' + canvas.flo_controller + '/' + canvas.flo_name + ': ' + total_time/10 + 'ms');
            }
        });
    });

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

                flo_canvas.resize_canvases();
            });
        }
    });

    flo_paint.initialise(add_action_event, perform_action);

    // All set up, let's go
    console.log('%c', 'background: url("' + base_url + '/png/Flo-Orb-small.png") no-repeat left center; background-size: 120px 142px; padding-left: 120px; padding-bottom: 71px; padding-top: 71px; line-height: 142px; font-size: 0%;"');
    console.log('%c=== F L O W B E T W E E N ===', 'font-family: monospace; font-weight: bold; font-size: 150%;');

    if (flo_paint.supports_pointer_events) {
        note('Will use pointer events for painting (modern browser)');
    } else if (flo_paint.supports_touch_events) {
        // Safari does not support pressure on OS X
        // Firefox does not support pressure on Windows
        note('Will use touch events for painting (browser is a bit out of date)');
    } else {
        note('Using mouse events for painting (browser is old, pressure sensitivity may not be available)');
    }

    inline_template_objects(document.getRootNode()).then(() => {
        reload_templates(document.getRootNode());
        new_session();
        enable_commands();
    });
}

///
/// For behavioural reasons we'd like svgs to be inline but for general work reasons
/// we'd like them to be objects. This lets us do an 'onload' on objects that causes
/// their content to be loaded in to the main document.
///
/// This is very helpful for making sure events go to the right place, and for using
/// CSS to style elements when they should be styled.
///
/// TODO: better yet would be to load this stuff into the actual template data, saving
/// the object load event every time.
///
let replace_object_with_content = (object_node) => {
    let parent      = object_node.parentNode;
    let document    = object_node.contentDocument;
    let content     = document.children[0];

    document.removeChild(content);
    parent.insertBefore(content, object_node.nextSibling);
    parent.removeChild(object_node);
};
