'use strict';

//   __ _                    _           _
//  / _| |___ ___ __ ___ _ _| |_ _ _ ___| |
// |  _| / _ \___/ _/ _ \ ' \  _| '_/ _ \ |
// |_| |_\___/   \__\___/_||_\__|_| \___/_|
//

/* exported flo_control */

let flo_control = (function () {
    ///
    /// Sets up a control as a slider, with the specified event as an update event
    ///
    let load_slider = (element) => {
        // Find the input element
        let input_element = element.getElementsByTagName('input')[0];

        // Retrieve the current property values from the object
        let flo_min_value   = element.flo_min_value || { 'Float': 0.0 };
        let flo_max_value   = element.flo_max_value || { 'Float': 100.0 };
        let flo_value       = element.flo_value || { 'Float': 0.0 };

        // Set the input range. We use a fixed range.
        input_element.min = 0.0;
        input_element.max = 1000.0;

        ///
        /// The 'focus' event is fired when the slider has focus
        ///
        function on_focus() {
            // This should generate the 'focus' event
            let input_handler = element.flo_was_focused || (() => {});

            // Pass on the event
            input_handler();
        }

        ///
        /// The 'input' event is fired while the user is changing the slider
        ///
        function on_input() {
            // If the node has the range property set, we'll return values in that range
            let flo_min = flo_min_value['Float'] || 0.0;
            let flo_max = flo_max_value['Float'] || 100.0;

            // We get a number 0-1000, change to fit in the range
            let value = (input_element.value/1000.0)*(flo_max-flo_min) + flo_min;

            // This is the editing event: if the node has an edit_value handler, this is where we send it
            let input_handler = element.flo_edit_value || (() => {});

            // Pass on the event (sliders generate float values)
            input_handler({ 'Float': value });
        }

        ///
        /// The 'change' event is fired when the user finishes dragging the slider to a new value
        ///
        function on_change() {
            // If the node has the range property set, we'll return values in that range
            let flo_min = flo_min_value['Float'] || 0.0;
            let flo_max = flo_max_value['Float'] || 100.0;

            // We get a number 0-1000, change to fit in the range
            let value = (input_element.value/1000.0)*(flo_max-flo_min) + flo_min;

            // This is the set event: if the node has an set_value handler, this is where we send it
            let input_handler = element.flo_set_value || (() => {});

            // Pass on the event (sliders generate float values)
            input_handler({ 'Float': value });
        }

        /// Updates the value of the slider to a particular value
        function set_value(new_property_value) {
            // Get the values that we're using
            let flo_min = flo_min_value['Float'] || 0.0;
            let flo_max = flo_max_value['Float'] || 100.0;
            let value   = new_property_value['Float'] || 0.0;

            // Change the value to be 0-1000
            value = ((value-flo_min)/(flo_max-flo_min))*1000.0;

            // Update the control
            input_element.value = value;
        }

        // Set the initial value
        set_value(element.flo_value || { 'Float': 0.0 });

        // Make the flo_min, flo_max and flo_value items dynamic properties by replacing them
        Object.defineProperty(element, 'flo_value', {
            get: () => flo_value,
            set: new_value => {
                if (new_value !== flo_value) {
                    flo_value = new_value;
                    set_value(new_value);
                }
            }
        });

        Object.defineProperty(element, 'flo_min_value', {
            get: () => flo_min_value,
            set: new_value => {
                if (new_value !== flo_min_value) {
                    flo_min_value = new_value;
                    set_value(flo_value);
                }
            }
        });

        Object.defineProperty(element, 'flo_max_value', {
            get: () => flo_max_value,
            set: new_value => {
                if (new_value !== flo_max_value) {
                    flo_max_value = new_value;
                    set_value(flo_value);
                }
            }
        });

        // Register event handlers
        input_element.addEventListener('focus', on_focus);
        input_element.addEventListener('input', on_input);
        input_element.addEventListener('change', on_change);
        input_element.addEventListener('blur', on_change);
    };

    ///
    /// Adds drag event handling to a node
    ///
    let on_drag = (node, add_action_event, start_drag, continue_drag, finish_drag, cancel_drag) => {
        start_drag          = start_drag || (() => {});
        continue_drag       = continue_drag || (() => {});
        finish_drag         = finish_drag || (() => {});
        cancel_drag         = cancel_drag || finish_drag;
        
        let start_client_x  = 0;
        let start_client_y  = 0;
        let start_drag_x    = 0;
        let start_drag_y    = 0;

        // Handles the mouse down event (this starts dragging immediately)
        // Starting dragging immediately prevents other kinds of actions
        let mouse_down = event => {
            // Only drag with the left mouse button
            if (event.button !== 0) {
                return;
            }

            // Add the event listeners to the document (so we receive everything that happens during the drag)
            document.addEventListener('mousemove', mouse_move, true);
            document.addEventListener('mouseup', mouse_up, true);

            // Stop the usual handling
            event.preventDefault();

            // Work out the location of the click in the target node
            let x = event.clientX;
            let y = event.clientY;

            start_client_x = x;
            start_client_y = y;

            // This slightly odd way of calculating node position partially deals 
            // with the fact that event.clientX does not take account of the 
            // transform but getBoundingClientRect does and we want the value
            // relative to the original rect
            let client_rect = node.parentNode.getBoundingClientRect();

            x -= client_rect.left + node.offsetLeft;
            y -= client_rect.top + node.offsetTop;

            start_drag_x = x;
            start_drag_y = y;
    
            // Flag that the drag event is starting
            start_drag(x, y);
        };

        // Handles the 'touch start' event (which also creates a drag effect)
        // Mouse down doesn't really work for dragging on touch devices (well, iOS devices anyway)
        let touch_start = event => {
            // Only drag with a single finger
            if (event.touches.length !== 1) {
                return;
            }

            // Add event handlers for the drag
            document.addEventListener('touchmove', touch_move, true);
            document.addEventListener('touchend', touch_end, true);
            document.addEventListener('touchcancel', touch_cancel, true);

            // Stop the default event (which will stop things like the annoying iOS bounce)
            event.preventDefault();

            // Work out the location of the touch in the target node
            let x = event.touches[0].clientX;
            let y = event.touches[0].clientY;

            start_client_x = x;
            start_client_y = y;

            let client_rect = node.parentNode.getBoundingClientRect();

            x -= client_rect.left + node.offsetLeft;
            y -= client_rect.top + node.offsetTop;

            start_drag_x = x;
            start_drag_y = y;

            start_drag(x, y);
        };

        // Moving the mouse continues the drag operation
        let mouse_move = event => {
            event.preventDefault();

            // Work out the location of the click in the target node
            let x = event.clientX;
            let y = event.clientY;

            x += (start_drag_x - start_client_x);
            y += (start_drag_y - start_client_y);

            // Continue the drag operation
            continue_drag(x, y);
        };

        // Releasing the mouse finishes the drag
        let mouse_up = event => {
            event.preventDefault();

            // Release the device
            document.removeEventListener('mousemove', mouse_move, true);
            document.removeEventListener('mouseup', mouse_up, true);

            // Dragging has finished
            finish_drag();
        };

        // Moving a touchpoint continues the drag operation
        let touch_move = event => {
            event.preventDefault();

            // Work out the location of the click in the target node
            let x = event.touches[0].clientX;
            let y = event.touches[0].clientY;

            x += (start_drag_x - start_client_x);
            y += (start_drag_y - start_client_y);

            // Continue the drag operation
            continue_drag(x, y);
        };

        // Releasing a touch ends the drag operation
        let touch_end = event => {
            event.preventDefault();

            // Release the device
            document.removeEventListener('touchmove', touch_move, true);
            document.removeEventListener('touchend', touch_end, true);
            document.removeEventListener('touchcancel', touch_cancel, true);

            // Dragging has finished
            finish_drag();
        };

        // Touch drags can wind up being cancelled (eg, by palm rejection)
        let touch_cancel = event => {
            event.preventDefault();

            // Release the device
            document.removeEventListener('touchmove', touch_move, true);
            document.removeEventListener('touchend', touch_end, true);
            document.removeEventListener('touchcancel', touch_cancel, true);

            // Dragging has been cancelled
            cancel_drag();
        };

        // Register for the mouse down event
        add_action_event(node, 'mousedown', mouse_down, false);
        add_action_event(node, 'touchstart', touch_start, true);
    };

    ///
    /// Sets up a control as a rotor
    ///
    let load_rotor = (rotor_node, add_action_event) => {
        // Retrieve the current property values from the object
        let flo_min_value   = rotor_node.flo_min_value || { 'Float': 0.0 };
        let flo_max_value   = rotor_node.flo_max_value || { 'Float': 100.0 };
        let flo_value       = rotor_node.flo_value || { 'Float': 0.0 };

        ///
        /// Updates the value displayed by the rotor
        ///
        let set_value = (new_value) => {
            // Get the values as floats
            let value   = new_value['Float'] || 0.0;
            let min     = flo_min_value['Float'] || 0.0;
            let max     = flo_max_value['Float'] || 100.0;

            // Angle goes from 0-360
            let angle = (value-min)/(max-min) * 360.0;

            // Transform through the node style
            rotor_node.style.transform = 'rotate(' + angle + 'deg)';
        };

        ///
        /// Computes the angle for a point near the node
        ///
        let angle_for_point = (x, y) => {
            // Assume that the node is a circle around its center
            let radius = rotor_node.clientWidth/2.0;

            x -= rotor_node.clientWidth/2.0;
            y -= rotor_node.clientHeight/2.0;

            if ((x*x + y*y) < (radius*radius)) {
                // If the point is within the main radius, then the angle is just the angle relative to the center
                return (Math.atan2(y, x) / (2*Math.PI) * 360.0);
            } else {
                // Really want to project a line onto the circle, then make the 
                // extra angle be the distance from the rotor. This has a 
                // similar effect but isn't quite as accurate.
                let angle           = (Math.atan2(y, x) / (2*Math.PI) * 360.0);
                let circumference   = Math.PI*2*radius;
                let extra_distance  = -x;
                if (x < -radius) {
                    extra_distance -= radius;
                } else if (x > radius) {
                    extra_distance += radius;
                } else {
                    extra_distance = 0;
                }

                return angle + ((extra_distance/circumference)*360);
            }
        };

        // Angle of an active dragging operation
        let drag_initial_angle  = 0.0;
        let initial_value       = flo_value;

        ///
        /// Event fired when a drag starts
        ///
        let start_drag = (x, y) => {
            // Store the initial angle and value
            drag_initial_angle  = angle_for_point(x, y);
            initial_value       = flo_value;
        };

        ///
        /// Event fired as a drag operation continues
        ///
        let continue_drag = (x, y) => {
            // Get the current values as floats
            let value   = initial_value['Float'] || 0.0;
            let min     = flo_min_value['Float'] || 0.0;
            let max     = flo_max_value['Float'] || 100.0;

            // Work out the angle difference
            let new_angle   = angle_for_point(x, y);
            let diff        = new_angle - drag_initial_angle;

            // Convert diff to a value over our range
            diff = (diff/360.0) * (max-min);

            // Compute a new value and clip to the range
            let new_value = value + diff;
            while (new_value > max) { new_value -= (max-min); }
            while (new_value < min) { new_value += (max-min); }

            // Set the new value
            flo_value = { 'Float': new_value };
            set_value(flo_value);

            // Fire the input event
            let input_handler = rotor_node.flo_edit_value || (() => {});
            input_handler({ 'Float': new_value });
        };

        ///
        /// Event fired as a drag operation finishes
        ///
        let end_drag = () => {
            // Fire the final event
            let input_handler = rotor_node.flo_set_value || (() => {});
            input_handler(flo_value);
        };

        ///
        /// Event fired if a drag operation is cancelled
        ///
        let cancel_drag = () => {
            // Final event resets the value
            flo_value = initial_value;

            let input_handler = rotor_node.flo_set_value || (() => {});
            input_handler(flo_value);
        };

        // Set the initial value for the rotor
        set_value(flo_value);

        // Make the flo_min, flo_max and flo_value items dynamic properties by replacing them
        Object.defineProperty(rotor_node, 'flo_value', {
            get: () => flo_value,
            set: new_value => {
                if (new_value !== flo_value) {
                    flo_value = new_value;
                    set_value(new_value);
                }
            }
        });

        Object.defineProperty(rotor_node, 'flo_min_value', {
            get: () => flo_min_value,
            set: new_value => {
                if (new_value !== flo_min_value) {
                    flo_min_value = new_value;
                    set_value(flo_value);
                }
            }
        });

        Object.defineProperty(rotor_node, 'flo_max_value', {
            get: () => flo_max_value,
            set: new_value => {
                if (new_value !== flo_max_value) {
                    flo_max_value = new_value;
                    set_value(flo_value);
                }
            }
        });

        // Register event handlers
        on_drag(rotor_node, add_action_event, start_drag, continue_drag, end_drag, cancel_drag);
    };

    ///
    /// Sets up a control as a popup
    ///
    let load_popup = (popup_node) => {
        // Set the initial state
        let is_open = popup_node.flo_popup_open || { 'Bool': false };

        // Function to set whether or not the popup is open or not
        function set_is_open(new_open) {
            is_open = new_open['Bool'] || false;

            popup_node.style.visibility = is_open ? 'visible' : 'hidden';
        }

        // Set the initial visibility
        set_is_open(is_open);

        // Replace the flo_popup_open property with one that updates the style
        Object.defineProperty(popup_node, 'flo_popup_open', {
            get: () => is_open,
            set: new_value => set_is_open(new_value)
        });
    };

    ///
    /// Determines if a document element is the root element or not
    ///
    let is_root = (element) => {
        if (!element) {
            return true;
        } else if (element.id === 'root') {
            return true;
        } else {
            return false;
        }
    };

    ///
    /// Determines the top-left coordinate of an element relative to its parent
    ///
    let position_in_parent = (element) => {
        return { 
            x: element.offsetLeft+element.clientLeft,
            y: element.offsetTop+element.clientTop
        };
    };

    ///
    /// Given a node, finds the coordinates of the total client area
    /// it can be placed in.
    ///
    let total_client_area = (element) => {
        // Start with the client area of the initial node
        let current_element = element;
        let current_rect    = { x1: 0, y1: 0, x2: element.clientWidth, y2: element.clientHeight };

        // Move upwards until we hit the root node
        while (!is_root(current_element)) {
            // Fetch the parent element
            let parent_element = current_element.parentNode;

            // Move the top-left corner of the rect to the parent position
            let position = position_in_parent(current_element);
            current_rect.x1 -= position.x;
            current_rect.y1 -= position.y;
            
            // Area corresponds to the client area of the parent element
            current_rect.x2 = current_rect.x1 + parent_element.clientWidth;
            current_rect.y2 = current_rect.y1 + parent_element.clientHeight;

            // Check the parent node next
            current_element = parent_element;
        }

        // current rect is now the 'screen' bounds for the element
        return current_rect;
    };

    ///
    /// Performs layout on a popup control
    ///
    let layout_popup = (popup_node, attributes) => {
        // Fetch the popup attributes
        let popup       = attributes.popup();

        let direction   = popup['Direction'] || 'Right';
        let size        = popup['Size'] || [100, 100];
        let offset      = popup['Offset'] || 0.0;

        // Get the area in which we're laying out our popup node
        let layout_area = total_client_area(popup_node.parentNode);

        // Get the area occupied by our parent node (popup is relative to this node)
        let parent_area = {
            x1: 0,
            y1: 0,
            x2: popup_node.parentNode.clientWidth,
            y2: popup_node.parentNode.clientHeight
        };

        // Create the 'preferred' area based on the direction and the parent position
        let target_area = {};

        switch (direction) {
        case 'OnTop':
            target_area = {
                x1: (parent_area.x1+parent_area.x2)/2.0 - size[0]/2.0,
                x2: (parent_area.x1+parent_area.x2)/2.0 + size[0]/2.0,
                y1: (parent_area.y1+parent_area.y2)/2.0 - size[1]/2.0,
                y2: (parent_area.y1+parent_area.y2)/2.0 - size[1]/2.0
            };
            break;

        case 'Left':
            target_area = {
                x1: parent_area.x1 - size[0] - offset,
                x2: parent_area.x1 - offset,
                y1: (parent_area.y1+parent_area.y2)/2.0 - size[1]/2.0,
                y2: (parent_area.y1+parent_area.y2)/2.0 - size[1]/2.0
            };
            break;

        case 'Right':
            target_area = {
                x1: parent_area.x2 + offset,
                x2: parent_area.x2 + size[0] + offset,
                y1: (parent_area.y1+parent_area.y2)/2.0 - size[1]/2.0,
                y2: (parent_area.y1+parent_area.y2)/2.0 - size[1]/2.0
            };
            break;

        case 'Above':
            target_area = {
                x1: (parent_area.x1+parent_area.x2)/2.0 - size[0]/2.0,
                x2: (parent_area.x1+parent_area.x2)/2.0 + size[0]/2.0,
                y1: parent_area.y1 - size[1] - offset,
                y2: parent_area.y1 - offset
            };
            break;

        case 'Below':
            target_area = {
                x1: (parent_area.x1+parent_area.x2)/2.0 - size[0]/2.0,
                x2: (parent_area.x1+parent_area.x2)/2.0 + size[0]/2.0,
                y1: parent_area.y2 + offset,
                y2: parent_area.y2 + size[1] + offset
            };
            break;

        case 'WindowCentered':
            target_area = {
                x1: (layout_area.x1+layout_area.x2)/2.0 - size[0]/2.0,
                x2: (layout_area.x1+layout_area.x2)/2.0 + size[0]/2.0,
                y1: (layout_area.y1+layout_area.y2)/2.0 - size[1]/2.0,
                y2: (layout_area.y1+layout_area.y2)/2.0 - size[1]/2.0
            };
            break;

        case 'WindowTop':
            target_area = {
                x1: (layout_area.x1+layout_area.x2)/2.0 - size[0]/2.0,
                x2: (layout_area.x1+layout_area.x2)/2.0 + size[0]/2.0,
                y1: layout_area.y1 + offset,
                y2: layout_area.y1 + offset + size[1]
            };
            break;
        }

        // Push the target area inside the window
        let translate = [0,0];
        if (target_area.x2 > layout_area.x2) { translate[0] = layout_area.x2 - target_area.x2; }
        if (target_area.y2 > layout_area.y2) { translate[1] = layout_area.y2 - target_area.y2; }
        if (target_area.x1 < layout_area.x1) { translate[0] = layout_area.x1 - target_area.x1; }
        if (target_area.y1 < layout_area.y1) { translate[1] = layout_area.y1 - target_area.y1; }

        target_area = {
            x1: target_area.x1+translate[0],
            x2: target_area.x2+translate[0],
            y1: target_area.y1+translate[1],
            y2: target_area.y2+translate[1]
        };

        // Position the beak
        let beak = popup_node.getElementsByTagName('deco-beak')[0];
        if (beak) {
            // Beak class is based on the direction
            beak.className = direction.toLowerCase();

            // Set the beak left/top position so that it centers on the parent element
            let parent_xpos = (target_area.x2 - target_area.x1)/2.0 - translate[0] - popup_node.clientLeft;
            let parent_ypos = (target_area.y2 - target_area.y1)/2.0 - translate[1] - popup_node.clientTop;

            switch (direction) {
            case 'Left':
            case 'Right':
                beak.style.top  = parent_ypos + 'px';
                beak.style.left = null;
                break;

            case 'Above':
            case 'Below':
                beak.style.left = parent_xpos + 'px';
                beak.style.top  = null;
                break;
            }
        }

        // Got the target area
        return target_area;
    };

    ///
    /// Recurses through a node and ensures that any item it contains that has a fixed scroll position stays where it is
    /// We use the margin CSS property for this, which makes it unavailable for other uses
    ///
    let fix_scroll_positions = (node) => {
        // Work out the offsets for this node
        let offset_left = node.scrollLeft;
        let offset_top  = node.scrollTop;

        // Iterate through the subtree and look for items with the flo-scroll-fix attribute
        let set_node_position = node => {
            let scroll_fix = node.getAttribute('flo-scroll-fix') || '';

            if (scroll_fix.indexOf('horiz') !== -1) {
                node.style.marginLeft = offset_left + 'px';
            }

            if (scroll_fix.indexOf('vert') !== -1) {
                node.style.marginTop = offset_top + 'px';
            }
        };

        let fix_child_positions = node => {
            [].slice.apply(node.children).forEach(child_node => {
                if (!child_node.getAttribute('flo-scroll-fix')) {
                    // Recurse into nodes that don't have the attribute set
                    fix_child_positions(child_node);
                } else {
                    // Set the positions of those that do
                    set_node_position(child_node);
                }
            });
        };

        fix_child_positions(node);
    };

    ///
    /// Sets up a control as a textbox
    ///
    let load_textbox = (node, add_action_event, on_property_change) => {
        // Fetch the values of the attributes that can be set for the text box
        let flo_text        = node.flo_text || { 'String': '' };
        let font_size       = node.getAttribute('flo-text-size') || null;
        let font_weight     = node.getAttribute('flo-text-weight') || null;
        let align           = node.getAttribute('flo-text-align') || null;

        // Set the initial text value
        let input           = node.getElementsByTagName('input')[0];
        let update_text     = (new_text_property) => {
            input.value = new_text_property['String'] || '';
        };

        update_text(flo_text);

        Object.defineProperty(node, 'flo_text', {
            get: () => flo_text,
            set: new_value => {
                if (new_value !== flo_text) {
                    flo_text = new_value;
                    update_text(new_value);
                }
            }
        });

        // Update the style attributes
        let style           = '';

        if (font_size)      { style += "font-size: " + font_size + "; " }
        if (font_weight)    { style += "font-weight: " + font_weight + "; " }
        if (align)          { style += "text-align: " + align + "; " }

        input.style         = style;

        // Bind the events for this node
        let has_focus = false;
        add_action_event(node, 'focus', event => {
            has_focus = true;
            if (node.flo_was_focused) {
                node.flo_was_focused();
            }
        });

        add_action_event(node, 'blur', event => {
            if (has_focus) {
                has_focus = false;

                if (node.flo_set_value) {
                    node.flo_set_value({ 'String': input.value || '' });
                }
            }
        });

        add_action_event(node, 'input', event => {
            if (node.flo_edit_value) {
                node.flo_edit_value({ 'String': input.value || '' });
            }
        });

        add_action_event(node, 'keydown', event => {
            if (!event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey) {
                if (event.key === 'Enter') {
                    event.preventDefault();
                    if (node.flo_set_value) {
                        node.flo_set_value({ 'String': input.value || '' });
                    }
                } else if (event.key === 'Escape') {
                    event.preventDefault();
                    if (node.flo_cancel_edit) {
                        node.flo_cancel_edit();
                    }
                }
            }
        });

        // Specify how the node gains focus
        node.flo_make_focused = () => { input.focus(); }
    };

    ///
    /// Sets up a control as a checkbox
    ///
    let load_checkbox = (node, add_action_event) => {
        // Get the input element
        let flo_value   = node.flo_value || { 'Bool': false };
        let input       = node.getElementsByTagName('input')[0];

        // Define a property to update the value of the textbox
        let update_value = (new_property_value) => {
            input.checked = new_property_value['Bool'] ? true : false;
        };

        update_value(flo_value);

        Object.defineProperty(node, 'flo_value', {
            get: () => flo_value,
            set: new_value => {
                if (new_value !== flo_value) {
                    flo_value = new_value;
                    update_value(new_value);
                }
            }
        });

        // Create 'SetValue' events when the checkbox value changes
        add_action_event(input, 'change', event => {
            if (node.flo_set_value) {
                node.flo_set_value({ 'Bool': input.checked ? true : false });
            }
        });

        add_action_event(node, 'focus', event => {
            if (node.flo_was_focused) {
                node.flo_was_focused();
            }
        });
    };

    return {
        load_slider:            load_slider,
        load_rotor:             load_rotor,
        load_popup:             load_popup,
        load_textbox:           load_textbox,
        load_checkbox:          load_checkbox,
        layout_popup:           layout_popup,
        on_drag:                on_drag,
        fix_scroll_positions:   fix_scroll_positions
    };
})();
