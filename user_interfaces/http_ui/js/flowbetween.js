'use strict';

//  ___ _            ___      _                       
// | __| |_____ __ _| _ ) ___| |___ __ _____ ___ _ _  
// | _|| / _ \ V  V / _ \/ -_)  _\ V  V / -_) -_) ' \ 
// |_| |_\___/\_/\_/|___/\___|\__|\_/\_/\___\___|_||_|
//                                                

/* exported flowbetween */
/* exported resize_svg_control */
/* exported replace_object_with_content */
/* global flo_canvas, flo_paint, flo_control */

function flowbetween(root_node) {
    /// The ID of the running session
    let running_session_id = '';

    // Control data, starting at the root node
    let root_control_data = null;

    // Nodes that are waiting for dismiss events
    let waiting_for_dismissal = [];

    // If an update is already running, this is the promise that will resolve when it's done
    let current_update_promise = Promise.resolve();

    // Promise that will resolve once the next update has completed
    let next_update_promise = Promise.resolve();

    // URL where the flowbetween session resides
    let target_url = '/flowbetween/session';

    // UTF encoder
    let utf8 = new TextEncoder('utf-8');

    // Maps websockets to session IDs
    let websocket_for_session = {};

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
        let templates           = {};
        let template_on_load    = {};
        let template_layout     = {};
        let template_resize     = {};

        ///
        /// Loads the UI templates for a particular DOM node
        ///
        let reload_templates = (root_node) => {
            // Clear the DOM nodes
            templates = {};

            // Find the template elements beneath the root node
            let root_templates = root_node.getElementsByTagName('TEMPLATE');

            // Each template can define the nodes we apply to flo-nodes, by example
            for (let template_number=0; template_number<root_templates.length; ++template_number) {
                let template_parent = root_templates[template_number].content;
                
                for (let nodeIndex=0; nodeIndex<template_parent.children.length; ++nodeIndex) {
                    let template_node   = template_parent.children[nodeIndex];
                    let template_name   = template_node.tagName.toLowerCase();
                    let template_class  = template_node.classList.item(0) || '';

                    templates[template_name]                    = templates[template_name] || {};
                    templates[template_name][template_class]    = [].slice.apply(template_node.children);
                    template_on_load[template_name]             = null;

                    // There is an onload attribute but it's set to null regardless for things that don't normally support onload
                    let on_load = template_node.getAttribute('onload');
                    if (on_load) {
                        template_on_load[template_name] = new Function('flowbetween', on_load);
                    }

                    // Template nodes can specify a custom layout algorithm if they want to override our standard one
                    let flo_layout = template_node.getAttribute('flo-layout');
                    if (flo_layout) {
                        let layout_fn = new Function('attributes', flo_layout);
                        flo_layout = (node, attributes) => layout_fn.apply(node, [attributes]);
                    } else {
                        flo_layout = null;
                    }

                    // They can also supply an on resize value
                    let flo_resize = template_node.getAttribute('flo-resize');
                    if (flo_resize) {
                        let resize_fn = new Function('', flo_resize);
                        flo_resize = (width, height, node) => resize_fn.apply([width, height, node]);
                    } else {
                        flo_resize = null;
                    }

                    template_layout[template_name] = flo_layout;
                    template_resize[template_name] = flo_resize;
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
            let template_name       = node.tagName.toLowerCase();
            let templates_for_node  = templates[template_name] || {};
            let template_for_node   = templates_for_node[''];
            let load_node           = template_on_load[template_name];
            let layout_node         = template_layout[template_name];
            let resize_node         = template_resize[template_name];

            // If the template has a subclass, use that
            node.classList.forEach((className) => { if (templates_for_node[className]) { template_for_node = templates_for_node[className]; } });

            if (template_for_node) {
                // Remove any nodes that might have been added to this node as part of a template
                get_decorative_subnodes(node)
                    .filter(decoration => decoration.getAttribute('flo-tmp'))
                    .forEach(decoration => node.removeChild(decoration));

                // Copy each template element into the document
                let new_nodes = template_for_node.map(template_node => document.importNode(template_node, true));

                // Add the nodes to this node
                let first_node = node.children.length > 0 ? node.children[0] : null;

                new_nodes.forEach(new_node => new_node.setAttribute('flo-tmp', ''))
                new_nodes.forEach(new_node => node.insertBefore(new_node, first_node));

                // Call the load function with our newly set up node
                if (load_node) {
                    // onload events get a 'flowbetween' parameter that can be used to access some internal functions
                    // add_action_event is an important one if they want to set up event handlers
                    // action events added during load are 'intrinsic' and stick around
                    let flowbetween = {
                        add_action_event:   add_intrinsic_action_event
                    };
                    load_node.apply(node, [flowbetween]);
                }

                // The layout engine will use the flo_layout property if it exists to lay out a node
                node.flo_layout = layout_node;

                if (resize_node) {
                    node.flo_resize = resize_node;
                }
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

        // padding retrieves the padding, if any
        let padding = () => {
            return get_attr('Padding');
        };

        // controller retrieves the name of the controller for this subtree
        let controller = () => {
            return get_attr('Controller');
        };

        // actions returns the list of actions that apply to this control
        let actions = () => {
            return get_attrs('Action');
        };

        // scrolls returns all of the scrolling attributes for this control
        let scrolls = () => {
            return get_attrs('Scroll');
        };

        // popup returns the list of popup attributes (combined into a single object)
        let popup = () => {
            let popups = get_attrs('Popup');
            return Object.assign.apply(null, [{}].concat(popups));
        };

        // Return an object that can be used to get information about these attributes
        return {
            all:            all,
            get_attr:       get_attr,
            subcomponents:  subcomponents,
            controller:     controller,
            actions:        actions,
            bounding_box:   bounding_box,
            padding:        padding,
            popup:          popup,
            scrolls:        scrolls
        };
    };

    ///
    /// Attempts to find the currently focused element (and its priority)
    ///
    let get_focused_element = () => {
        // Get the currently active element
        let active  = document.activeElement;
        let focused = active;

        // Move up the tree until we find a node that supplies a focus level (focus levels apply to all child element)
        while (focused) {
            // Flo nodes all have a focus level function we can call
            if (focused.flo_focus_level) {
                // Query the node for its focus level
                let focus_level = focused.flo_focus_level();

                if (focus_level !== null) {
                    // This node can be considered to have focus. Nodes with a 'null' focus level pass focus through
                    return {
                        node:       focused,
                        priority:   focus_level
                    }
                }
            }

            // Move up the tree
            focused = focused.parentElement;
        }

        // No node has focus
        return {
            node:       null,
            priority:   0
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
    /// Finds the flo node at the specified address
    ///
    let node_at_address = (address) => {
        let current_node = root_node;

        // The root node is the div containing the document. The root control node should be it's only child.
        current_node = get_flo_subnodes(current_node)[0];

        // Follow the address
        address.forEach(index => current_node = get_flo_subnodes(current_node)[index]);

        // This is the node at this address
        return current_node;
    };

    ///
    /// Finds the control data at a particular address, and its parent node
    ///
    let data_at_address = (address) => {
        let parent_node             = null;
        let current_data            = root_control_data;
        let controller_path         = [];

        address.forEach(index => {
            // Find the data for the subcomponent at this address
            let attributes  = get_attributes(current_data);
            parent_node     = current_data;
            current_data    = attributes.subcomponents()[index];

            // If this component has a controller associated with it, that's the controller for any subcomponents
            let controller = attributes.controller();
            if (controller) {
                controller_path.push(controller);
            }
        });

        return {
            data:               current_data, 
            parent:             parent_node,
            controller_path:    controller_path
        };
    };

    ///
    /// Visits the flo items in the DOM, passing in attributes from
    /// the appropriate control data sections
    ///
    let visit_dom = (dom_node, control_data, visit_node, initial_controller_path) => {
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
        visit_internal(dom_node, control_data, visit_node, initial_controller_path || []);
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

        case 'Floating': {
            let offset      = next_pos_desc[pos_type][1];

            return offset;
        }

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
    let layout_subcomponents = (parent_node, attributes, controller_path) => {
        let subcomponents   = attributes.subcomponents();
        let subnodes        = get_flo_subnodes(parent_node);
        let positions       = [];
        let total_width     = parent_node.clientWidth;
        let total_height    = parent_node.clientHeight;

        if (subcomponents === null) {
            return;
        }

        // Stop any layout that's happening for this node already
        if (parent_node.flo_unbind_layout) {
            parent_node.flo_unbind_layout();
            parent_node.flo_unbind_layout = null;
        }

        // Scrolling containers might specify their own minimum size
        if (parent_node.tagName.toLowerCase() === 'flo-scrolling') {
            let scrolls = attributes.scrolls();

            scrolls.forEach(scroll_attr => {
                if (scroll_attr['MinimumContentSize']) {
                    if (scroll_attr['MinimumContentSize'][0] > total_width)     { total_width   = scroll_attr['MinimumContentSize'][0]; }
                    if (scroll_attr['MinimumContentSize'][1] > total_height)    { total_height  = scroll_attr['MinimumContentSize'][1]; }
                }
            });
        }

        // Take account of the padding
        let padding = attributes.padding() || { top: 0, left: 0, right: 0, bottom: 0 };

        total_width     -= padding.left+padding.right;
        total_height    -= padding.top+padding.bottom;

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
            let element         = subnodes[node_index];
            let component       = subcomponents[node_index];
            let attributes      = get_attributes(component);
            let bounding_box    = attributes.bounding_box() || default_bounding_box;
            let layout_override = element.flo_layout;

            element.flo_prev_width  = element.clientWidth;
            element.flo_prev_height = element.clientHeight;

            if (!layout_override) {
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
            } else {
                // Node has custom layout behaviour: call the flo_layout property to get the position
                positions.push(layout_override(element, attributes));
            }
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

                if (!component.flo_layout) {
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
        }

        // Elements with an 'AtPosition' value are 'floating': we update their position based on the value of the property
        // The need to do binding here complicates things quite a lot: the usual thing 
        // that happens here is that we just set the left, top, width or height properties
        // to the value stored in the attribute. When it's bound we update it whenever
        // it changes (without triggering a new layout)
        let make_floating = (initial_value, property, set_value) => {
            return on_property_change(controller_path, property, (value) => { set_value((value['Float'] || 0) + initial_value); return true; });
        };
        let remove_actions = [];

        // Either sets the position directly, or generates an action to track the specified property
        let bind_property_position = (node_index, initial_value, get_position, set_position) => {
            // Fetch the position for this node
            let component       = subcomponents[node_index];
            let bounding_box    = get_attributes(component).bounding_box() || default_bounding_box;
            let position        = get_position(bounding_box);

            if (position && position['Floating']) {
                // This is a floating node: bind to its property value
                remove_actions.push(make_floating(initial_value, position['Floating'][0], set_position));
            } else {
                // Just a standard node: set to the initial position and leave it
                set_position(initial_value);
            }
        };

        // Final pass: finish the layout
        for (let node_index=0; node_index<subcomponents.length; ++node_index) {
            let element         = subnodes[node_index];
            let pos             = positions[node_index];

            // Set the positions, performing viewmodel binding if necessary
            let x1 = pos.x1+padding.left;
            let y1 = pos.y1+padding.top;
            let x2 = pos.x2+padding.left;
            let y2 = pos.y2+padding.top;

            bind_property_position(node_index, pos.x1,  (bounds) => bounds.x1, (pos) => { 
                x1 = pos+padding.left;
                element.style.left      = x1 + 'px';
                element.style.width     = (x2-x1) + 'px';
            });
            bind_property_position(node_index, pos.x2,  (bounds) => bounds.x2, (pos) => { 
                x2 = pos+padding.left; 
                element.style.width     = (x2-x1) + 'px';
            });
            bind_property_position(node_index, pos.y1,  (bounds) => bounds.y1, (pos) => {
                y1 = pos+padding.top; 
                element.style.top       = y1 + 'px'; 
                element.style.height    = (y2-y1) + 'px';
            });
            bind_property_position(node_index, pos.y2,  (bounds) => bounds.y2, (pos) => { 
                y2 = pos+padding.top;
                element.style.height    = (y2-y1) + 'px'; 
            });

            // If the node has an on resize property, then call that after laying it out
            let on_resize = element.flo_resize;
            if (on_resize && (element.flo_prev_width !== element.clientWidth || element.flo_prev_height !== element.clientHeight)) {
                on_resize(element.clientWidth, element.clientHeight, element);
            }
        }

        // Make note of the remove actions if there were any
        if (remove_actions.length > 0) {
            parent_node.flo_unbind_layout = () => {
                remove_actions.forEach(remove_item => remove_item());
                remove_actions = [];
            };
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
    /// Adds an action event that's 'intrinsic' to the node (kept even when
    /// we want to rewire the events)
    ///
    let add_intrinsic_action_event = (node, event_name, handler, options) => {
        // Works just like add_action_event...
        add_action_event(node, event_name, handler, options);
        
        // ...except we also record a function for re-registering these
        let also_intrinsic = node.flo_register_intrinsic_events;
        node.flo_register_intrinsic_events = () => {
            if (also_intrinsic) {
                also_intrinsic();
            }

            add_action_event(node, event_name, handler, options);
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
            var handle_event    = false;
            var prevent_default = false;

            if (event.eventPhase !== Event.CAPTURING_PHASE) {
                // Always handle events at source
                handle_event    = true;
                prevent_default = true;
            } else {
                // Capture when there's not an existing click handler closer to the start
                var path = event.composedPath();
                for (var path_index = 0; path_index < path.length; ++path_index) {
                    var node_in_path = path[path_index];

                    if (node_in_path['flo_event_click']) {
                        if (node_in_path === node) {
                            handle_event = true;
                        }
                        break;
                    }
                }
            }

            if (handle_event) { 
                if (prevent_default) { event.preventDefault(); }
                event.stopPropagation();
                note('Click ' + action_name + ' --> ' + controller_path);

                perform_action(controller_path, action_name, null);
            }
        }, true);

        add_action_event(node, 'touchstart', event => {
            var handle_event    = false;
            var prevent_default = false;

            if (event.eventPhase !== Event.CAPTURING_PHASE) {
                // Always handle events at source
                handle_event    = true;
                prevent_default = true;
            } else {
                // Capture when there's not an existing click handler closer to the start
                var path = event.composedPath();
                for (var path_index = 0; path_index < path.length; ++path_index) {
                    var node_in_path = path[path_index];

                    if (node_in_path['flo_event_touchstart']) {
                        if (node_in_path === node) {
                            handle_event = true;
                        }
                        break;
                    }
                }
            }

            if (event.touches.length === 1 && handle_event) {
                if (prevent_default) { event.preventDefault(); }
                event.stopPropagation();
                note('Click (touch) ' + action_name + ' --> ' + controller_path);

                perform_action(controller_path, action_name, null);
            }
        }, true);
    };

    ///
    /// Wires up a drag action to a node
    ///
    let wire_drag = (action_name, node, controller_path) => {
        // Last known drag coordinates
        let start_x = 0;
        let start_y = 0;
        let last_x  = 0;
        let last_y  = 0;

        // Drag operation is starting
        let start_drag = (x, y) => {
            start_x = last_x = x;
            start_y = last_y = y;

            perform_action(controller_path, action_name, { 'Drag': [ 'Start', [start_x, start_y], [x, y]] });
        };

        // Drag operation continues
        let continue_drag = (x, y) => {
            last_x = x;
            last_y = y;

            perform_action(controller_path, action_name, { 'Drag': [ 'Drag', [start_x, start_y], [x, y]] });
        };

        // Drag operation finishes
        let finish_drag = () => {
            perform_action(controller_path, action_name, { 'Drag': [ 'Finish', [start_x, start_y], [last_x, last_y]] });
        };

        // Drag operation got cancelled
        let cancel_drag = () => {
            perform_action(controller_path, action_name, { 'Drag': [ 'Cancel', [start_x, start_y], [start_x, start_y]] });
        };

        // Wire up the event
        flo_control.on_drag(node, add_action_event, start_drag, continue_drag, finish_drag, cancel_drag);
    };

    ///
    /// Rewires any intrinsic events that might have been removed by a
    /// call to remove_action_events_from_node
    ///
    let rewire_intrinsic_events = (node) => {
        let register_intrinsic = node.flo_register_intrinsic_events;
        if (register_intrinsic) {
            register_intrinsic();
        }
    };

    ///
    /// Wires up an action to a node
    ///
    let wire_action = (action, node, controller_path) => {
        let remove_action = null;

        // If this node is already wired up, remove the events we added
        remove_action_events_from_node(node);
        rewire_intrinsic_events(node);

        // Store the actions for this event
        let action_type = action[0];
        let action_name = action[1];

        if (action_type === 'Click') {
            wire_click(action_name, node, controller_path);

        } else if (action_type['VirtualScroll']) {
            wire_virtual_scroll(action_name, node, controller_path, action_type['VirtualScroll'][0], action_type['VirtualScroll'][1]);

        } else if (action_type['Paint']) {
            flo_paint.wire_paint(action_type['Paint'], action_name, node, controller_path);

        } else if (action_type === 'Drag') {
            wire_drag(action_name, node, controller_path);

        } else if (action_type === 'Focused') {
            node.flo_was_focused = new_property_value => perform_action(controller_path, action_name, null);

        } else if (action_type === 'EditValue') {
            node.flo_edit_value = new_property_value => perform_action(controller_path, action_name, { 'Value': new_property_value });

        } else if (action_type === 'SetValue') {
            node.flo_set_value = new_property_value => perform_action(controller_path, action_name, { 'Value': new_property_value });

        } else if (action_type === 'CancelEdit') {
            node.flo_cancel_edit = new_property_value => perform_action(controller_path, action_name, null);

        } else if (action_type === 'Dismiss') {
            node.flo_dismiss = () => perform_action(controller_path, action_name, null);

            waiting_for_dismissal.push(node);
            remove_action = () => {
                node.flo_dismiss        = null;
                waiting_for_dismissal   = waiting_for_dismissal.filter(dismiss_node => dismiss_node !== node);
            };

        } else {
            warn('Unknown action type: ' + action_type);
        }

        // If the action requires unwiring, store how in the node
        if (remove_action) {
            let remove_more = node.flo_remove_actions;

            node.flo_remove_actions = () => {
                if (remove_action) {
                    remove_action();
                    if (remove_more) {
                        remove_more();
                    }

                    remove_action   = null;
                    remove_more     = null;
                }
            };
        }
    };

    ///
    /// Wires up events for a component
    ///
    let wire_events = (node, attributes, controller_path) => {
        // Remove existing events, if any
        if (node.flo_remove_actions) {
            remove_action_events_from_node(node);
            rewire_intrinsic_events(node);
        }

        // Fetch and wire actions
        let actions = attributes.actions();

        if (actions) {
            actions.forEach(action => wire_action(action, node, controller_path));
        }
    };

    ///
    /// Fires a virtual scroll event for a node (useful when the node is scrolled or when the node is resized)
    ///
    let virtual_scroll_node = (controller_path, action_name, node, grid_x, grid_y) => {
        // Fetch the coordinates from the node
        let offset_x    = node.scrollLeft;
        let offset_y    = node.scrollTop;
        let width       = node.clientWidth;
        let height      = node.clientHeight;

        // Convert into grid coords
        let xpos        = Math.floor(offset_x / grid_x);
        let ypos        = Math.floor(offset_y / grid_y);
        let grid_width  = Math.ceil((width / grid_x)+0.5);
        let grid_height = Math.ceil((height / grid_y)+0.5);

        let changed     = false;
        let last_pos    = node.flo_virtual_scroll;

        if (!last_pos) {
            changed = true;
        } else {
            changed  = last_pos.xpos !== xpos 
                    || last_pos.ypos !== ypos 
                    || last_pos.grid_width !== grid_width
                    || last_pos.grid_height !== grid_height;
        }

        if (changed) {
            // Store the current position
            node.flo_virtual_scroll = {
                xpos, ypos, grid_width, grid_height
            };

            // Fire the event
            perform_action(controller_path, action_name, { 'VirtualScroll': [ [xpos, ypos], [grid_width, grid_height] ] });
        }
    };

    ///
    /// Wires a node for the virtual scroll event
    ///
    let wire_virtual_scroll = (action_name, node, controller_path, grid_x, grid_y) => {
        // Function to fire when the node scrolls
        let will_scroll = false;
        let scroll_now  = () => virtual_scroll_node(controller_path, action_name, node, grid_x, grid_y);

        let on_scroll   = () => {
            if (!will_scroll) {
                will_scroll = true;
                requestAnimationFrame(() => {
                    will_scroll = false;
                    scroll_now();
                });
            }
        };

        // Scroll the node whenever the scroll event is fired
        add_action_event(node, 'scroll', () => on_scroll());

        // Also scroll whenever the node's size changes
        let more_resize = node.flo_resize;
        node.flo_resize = (width, height, element) => {
            on_scroll();
            if (more_resize) {
                more_resize(width, height, element);
            }
        };

        // Also scroll right now
        on_scroll();
    };

    ///
    /// Binds a single attribute to a node
    ///
    let bind_viewmodel_attribute = (node, attribute, controller_path) => {
        let remove_action = null;

        let node_binding = node.flo_bind_viewmodel;
        if (node_binding && (remove_action = node_binding(node, attribute, controller_path, on_property_change))) {
            // Custom controls might have their own binding, which they can set up by adding a flo_bind_viewmodel attribute to the binding

        } else if (attribute['Selected']) {
            // The selected property updates the node class
            remove_action = on_property_change(controller_path, attribute['Selected'], is_selected => {
                if (is_selected['Bool']) {
                    add_class(node, 'selected');
                } else {
                    remove_class(node, 'selected');
                }

                return true;
            });

        } else if (attribute['Enabled']) {
            // The enabled property updates the node class
            remove_action = on_property_change(controller_path, attribute['Enabled'], is_enabled => {
                if (is_enabled['Bool']) {
                    add_class(node, 'enabled');
                    remove_class(node, 'disabled');
                } else {
                    remove_class(node, 'enabled');
                    add_class(node, 'disabled');
                }

                return true;
            });

        } else if (attribute['Badged']) {
            // The badged property updates the node class
            remove_action = on_property_change(controller_path, attribute['Badged'], is_badged => {
                if (is_badged['Bool']) {
                    add_class(node, 'badged');
                } else {
                    remove_class(node, 'badged');
                }

                return true;
            });

        } else if (attribute['Value']) {
            // Value just updates the flo_value property
            remove_action = on_property_change(controller_path, attribute['Value'], new_value => {
                node.flo_value = new_value;
                return true;
            });

        } else if (attribute['Text']) {
            // Value just updates the flo_text property
            remove_action = on_property_change(controller_path, attribute['Text'], new_value => {
                node.flo_text = new_value;
                return true;
            });

        } else if (attribute['Range']) {
            // Range updates the min value and max value properties
            let remove_action1 = on_property_change(controller_path, attribute['Range'][0], new_value => {
                node.flo_min_value = new_value;
                return true;
            });
            let remove_action2 = on_property_change(controller_path, attribute['Range'][1], new_value => {
                node.flo_max_value = new_value;
                return true;
            });

            remove_action = () => {
                remove_action1();
                remove_action2();
            };

        } else if (attribute['Popup'] && attribute['Popup']['IsOpen']) {
            // Updates the popup open property
            remove_action = on_property_change(controller_path, attribute['Popup']['IsOpen'], new_value => {
                node.flo_popup_open = new_value;
                return true;
            });

        } else if (attribute['Scroll']) {
            let scroll = attribute['Scroll'];

            if (scroll['MinimumContentSize']) {
                let canvas_deco             = node.getElementsByTagName('deco-scroll-canvas')[0];
                canvas_deco.style.width     = scroll['MinimumContentSize'][0] + 'px';
                canvas_deco.style.height    = scroll['MinimumContentSize'][1] + 'px';
            }

        } else if (attribute['FocusPriority']) {
            // Updates the focus priority for this node
            remove_action = on_property_change(controller_path, attribute['FocusPriority'], focus_priority => {
                let new_priority = 0;

                // Get the priority of this node
                if (focus_priority['Int']) {
                    new_priority = focus_priority['Int'];
                } else if (focus_priority['Float']) {
                    new_priority = focus_priority['Float'];
                }

                // Update the returned priority for this node
                node.flo_focus_level = () => new_priority;

                // Move focus here if it's greater than the priority of the node that currently has focus
                let current_focus = get_focused_element();
                if (current_focus.priority < new_priority && node.flo_make_focused) {
                    requestAnimationFrame(() => node.flo_make_focused());
                }
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
    /// Removes any events and other attachments from a node and its children
    ///
    let unwire_node = (node) => {
        // Unwires a single node
        let unwire = (node) => {
            let unbind_viewmodel = node.flo_unbind_viewmodel;
            if (unbind_viewmodel) { 
                unbind_viewmodel(); 
            }

            if (node.flo_remove_actions) {
                remove_action_events_from_node(node);
            }

            let unbind_layout = node.flo_unbind_layout;
            if (unbind_layout) {
                unbind_layout();
            }

            if (node.tagName.toLowerCase() === 'flo-canvas') {
                flo_canvas.stop(node); 
            }        
        };

        // Recursively unwires a node
        let unwire_recursive = (node) => {
            if (node) {
                // Unwire the subnodes
                get_flo_subnodes(node).forEach((sub_node) => unwire_recursive(sub_node));

                // Unwire the node itself
                unwire(node);
            }
        };

        // Start at the first node
        unwire_recursive(node);
    };

    ///
    /// Sends dismiss events to anything that's waiting for one and is
    /// not a parent of the specified node.
    ///
    let dismiss_others = (event_node) => {
        // Nothing to do if nothing is waiting for a dismiss event
        if (waiting_for_dismissal.length <= 0) {
            return;
        }

        // The target nodes are the nodes along the path for this event
        // If this is an interaction with a dismissable control, it won't be dismissed
        let target_nodes = [];
        let current_node = event_node;

        while (current_node !== null && current_node !== root_node) {
            target_nodes.push(current_node);
            current_node = current_node.parentNode;
        }

        // Dismiss any waiting node that isn't in the 'target' list
        let to_dismiss = waiting_for_dismissal.filter(dismiss_node => {
            if (target_nodes.find(node => node === dismiss_node)) {
                // Node is a child of the item that wants the dismiss event (doesn't cause a dismiss)
                return false;
            } else if (target_nodes.find(node => node === dismiss_node.parentNode)) {
                // Node is the parent of the item that wants the dismiss event
                return false;
            } else {
                return true;
            }
        });

        // When the event was wired, the flo_dismiss property was added
        to_dismiss.forEach(dismiss_node => {
            dismiss_node.flo_dismiss();
        });
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

            updates.forEach(update => {
                let new_property        = update['NewProperty'];
                let property_changed    = update['PropertyChanged'];

                if (new_property) {
                    viewmodel.keys[new_property[0]] = new_property[1];
                } else if (property_changed) {
                    viewmodel.keys[property_changed[0]] = property_changed[1];
                } else {
                    console.error('Viewmodel update', update, 'was not understood');
                }
            });

            // Notify anything that's listening of the changes
            updates.forEach(update => {
                let key_value   = update['NewProperty'] || update['PropertyChanged'];
                let key         = key_value[0];
                let new_value   = key_value[1];
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
                    note('Double removal of action for ' + key + ' on controller ' + controller_path);
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
    /// Resolves the 'next update' promise
    ///
    let resolve_next_update = (function () {
        // on_resolve will resolve the update promise. 
        let on_resolve          = null;
        let create_new_promise  = () => {
            next_update_promise = new Promise((resolve) => {
                on_resolve = () => resolve();
            });
        };

        // Create the initial 'next_update' promise
        create_new_promise();

        // Function resolves the existing promise and creates a new one
        return () => {
            on_resolve();
            create_new_promise();
        };
    })();

    ///
    /// Connects to a websocket running on a different port on the same server
    ///
    let connect_websocket = (websocket_port, session_id) => {
        websocket_port = websocket_port || doc_url.port;

        var connect = new Promise((resolve) => {
            // Construct the WS URL from the document URL
            let ws_base_url     = 'ws://' + doc_url.hostname + ':' + websocket_port;
            let ws_session_url  = ws_base_url + '/ws/' + session_id;

            note('Connecting to websocket at ' + ws_session_url);

            // Connect the websocket
            let websocket = new WebSocket(ws_session_url);

            // Add event handlers for it
            websocket.addEventListener('message', (event) => {
                // Decode the updates from the message
                let updates = JSON.parse(event.data);

                // Dispatch them
                current_update_promise = current_update_promise.then(() => dispatch_updates(updates))
                    .then(() => {
                        current_update_promise = Promise.resolve();
                    })
                    .catch((err) => {
                        current_update_promise = Promise.resolve();
                        error('Request failed.', err);
                    });
            });

            websocket.addEventListener('open', () => {
                note('Websocket for session ' + session_id + ' is connected');

                // Register this as the socket for this session
                websocket_for_session[session_id] = websocket;

                // Resolve the promise
                resolve();
            });

            websocket.addEventListener('error', (event) => {
                // Revert to the standard UI handler if we get an error
                error('Session ' + session_id + ' suffered a websocket error: ', event);
                websocket_for_session[session_id] = null;
            });
        });

        return connect;
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
        visit_dom(dom_node, control_data, (node, attributes, controller_path) => layout_subcomponents(node, attributes, controller_path));
    };

    ///
    /// Given a node and its control data, wires up any events
    ///
    let set_tree_attributes = (dom_node, control_data, initial_controller_path) => {
        visit_dom(dom_node, control_data, (node, attributes, controller_path) => {
            // Store the attributes for this node for convenience
            node.flo = {
                controller: controller_path,
                attributes: attributes
            };

            // Default focus level for an element is 0
            node.flo_focus_level    = () => 0;

            // Default focus action is 'do nothing'
            node.flo_make_focused   = () => {};
        }, initial_controller_path);
    };

    ///
    /// Given a node and its control data, wires up any events
    ///
    let wire_tree = (dom_node, control_data, initial_controller_path) => {
        visit_dom(dom_node, control_data, (node, attributes, controller_path) => {
            // Attach any events that this node might require
            wire_events(node, attributes, controller_path);
        }, initial_controller_path);
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
    let bind_viewmodel_to_tree = (dom_node, control_data, initial_controller_path) => {
        visit_dom(dom_node, control_data, (node, attributes, controller_path) => bind_viewmodel(node, attributes, controller_path), initial_controller_path);
    };

    ///
    /// The entire UI HTML should be replaced with a new version
    ///
    let on_new_html = (new_user_interface_html, property_tree) => {
        note('Replacing user interface');

        return new Promise((resolve) => {
            let root = get_root();
            
            // Update the DOM
            root.innerHTML      = new_user_interface_html;
            root_control_data   = property_tree;

            // Perform initial layout
            set_tree_attributes(get_flo_subnodes(root)[0], root_control_data);
            apply_templates_to_tree(get_flo_subnodes(root)[0], root_control_data);
            bind_viewmodel_to_tree(get_flo_subnodes(root)[0], root_control_data);
            wire_tree(get_flo_subnodes(root)[0], root_control_data);
            layout_tree(get_flo_subnodes(root)[0], root_control_data);

            resolve();
        });
    };

    ///
    /// A portion of the HTML tree has been updated
    ///
    let on_update_html = (updates) => {
        
        note('Updating HTML');

        return new Promise((resolve) => {
            // Find the original nodes for each update
            updates.forEach(update => {
                update.original_node    = node_at_address(update.address);
                update.original_data    = data_at_address(update.address);
            });

            // Unwire the original DOM
            updates.forEach(update => {
                unwire_node(update.original_node);
            });

            // Replace the data for each element involved in the update
            updates.forEach(update => {
                let address = update.address;

                if (address.length === 0) {
                    // 0-length addresses replace the root node
                    root_control_data = update.ui_tree;
                } else {
                    // Other attribute replace the subcomponents
                    let node_index  = address[address.length-1];
                    let parent_node = update.original_data.parent;
                    let attributes  = get_attributes(parent_node);

                    attributes.subcomponents()[node_index] = update.ui_tree;
                }
            });

            // Replace the HTML for each element involved in the update
            updates.forEach(update => {
                // Generate the replacement HTML element
                let template        = document.createElement('template');
                template.innerHTML  = update.new_html;
                let new_element     = template.content.childNodes[0];

                // Replace the original node
                let parent_node     = update.original_node.parentNode;
                if (parent_node) {
                    parent_node.replaceChild(new_element, update.original_node);
                }

                update.new_element  = new_element;
            });

            // Reformat/bind/wire the new HTML
            updates.forEach(update => {
                set_tree_attributes(update.new_element, update.ui_tree, update.original_data.controller_path);
                apply_templates_to_tree(update.new_element, update.ui_tree, update.original_data.controller_path);
                bind_viewmodel_to_tree(update.new_element, update.ui_tree, update.original_data.controller_path);
                wire_tree(update.new_element, update.ui_tree, update.original_data.controller_path);
            });

            // Update the layout of everything once we're done
            if (root_control_data) {
                layout_tree(get_flo_subnodes(get_root())[0], root_control_data);
            }

            // Tidy canvases if necessary
            flo_canvas.update_canvas_map();

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

                } else if (update['WebsocketPort']) {

                    current_promise = current_promise
                        .then(() => connect_websocket(update['WebsocketPort'], running_session_id));

                } else if (update === 'WebsocketSamePort') {

                    current_promise = current_promise
                        .then(() => connect_websocket(null, running_session_id));

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

                } else if (update['UpdateHtml']) {

                    let updates = update['UpdateHtml'];

                    current_promise = current_promise
                        .then(() => on_update_html(updates));

                } else {
                    warn('Unknown update type', Object.keys(update)[0], update);
                }
            });

            // Notify that the update has completed
            resolve_next_update();

            return current_promise;
        };
    })();

    ///
    /// Sends a request to the session URI and processes the result
    ///
    let send_request = (function() {
        // Flag that sets if we log the events to the developer console
        let show_requests       = false;
        add_command('show_server_requests', 'Log the requests sent to the server', () => show_requests = true);
        add_command('hide_server_requests', 'Stop requests sent to the server', () => show_requests = false);

        // Events waiting to be sent
        let pending_events      = [];

        // Set to true when we're going to send the events we gathered
        let sending_events      = null;

        // Promise sending the last set of events
        let last_promise        = null;

        // Waiting for the previous update
        let previous_update    = null;

        return (request) => {
            let session_id  = request.session_id;
            let events      = request.events;

            if (session_id && websocket_for_session[session_id]) {
                if (show_requests && sending_events && pending_events.length == 0) {
                    note('Queuing events while we wait for the server');
                }

                // Add the events to the pending list
                pending_events = pending_events.concat(events);

                // Events are collated and sent at the start of the next frame
                if (!sending_events) {
                    sending_events = true;

                    // Promise that will complete after the next animation frame
                    var animation_frame;

                    if (previous_update != null) {
                        // Wait for the previous update, then request the animation frame
                        let wait_for_update = previous_update;
                        animation_frame = wait_for_update.then(() => new Promise((resolve) => {
                            requestAnimationFrame(() => resolve());
                        }));
                    } else {
                        // Just wait for the animation frame
                        animation_frame = new Promise((resolve) => {
                            requestAnimationFrame(() => resolve());
                        });
                    }

                    // The update that will follow this one is what's current set as the next update
                    previous_update = next_update_promise;

                    // Send to the websocket after the event
                    let promise = animation_frame.then(() => {
                        // Get the events we're going to send and reset the state
                        sending_events = null;
                        let events     = pending_events;
                        pending_events = [];

                        if (show_requests) {
                            console.log('Sending request', events);
                        }

                        // Send to the websocket
                        let websocket = websocket_for_session[session_id];

                        events.unshift('SuspendUpdates');
                        events.push('Tick');
                        events.push('ResumeUpdates');
                        websocket.send(JSON.stringify(events));
                    });

                    last_promise     = promise;

                    // Resolve once the update from this message is generated
                    let next_update = next_update_promise;
                    return promise.then(() => next_update);
                } else {
                    // Resolve when the pending update resolves
                    return last_promise;
                }
            } else {
                return retry(() => http_post(request), () => warn('UI request failed - retrying'))
                    .then((response) => response_to_object(response))
                    .then((ui_request) => dispatch_updates(ui_request.updates))
                    .catch((err) => {
                        error('Request failed.', err);
                    });
            }
        };
    })();

    ///
    /// Makes a request to refresh the current state of the UI
    ///
    let refresh_ui = () => {
        note('Requesting UI refresh');
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
    /// ===== DEBUGGING AND INTROSPECTION
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

    // Interacting outside 'dismiss' nodes should fire the 'dismiss' event
    root_node.addEventListener('pointerdown', ev => {
        dismiss_others(ev.target);
    }, {
        capture: true,
        passive: true
    });

    root_node.addEventListener('touchstart', ev => {
        dismiss_others(ev.target);
    }, {
        capture: true,
        passive: true
    });

    root_node.addEventListener('mousedown', ev => {
        dismiss_others(ev.target);
    }, {
        capture: true,
        passive: true
    });

    // Prepare for painting
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
/// For controls with an SVG element underneath them, this will set the width and height
/// attributes to the width and height of the control when it is resized
///
let resize_svg_control = (width, height, parent_element) => {
    let svg = parent_element.getElementsByTagName('svg');
    for (let element_id=0; element_id<svg.length; ++element_id) {
        svg[element_id].setAttribute('width', width);
        svg[element_id].setAttribute('height', height);
    }
};

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
