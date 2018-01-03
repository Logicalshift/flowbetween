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

        /// The 'input' event is fired while the user is changing the slider
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

        /// The 'change' event is fired when the user 
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
        input_element.addEventListener('input', on_input);
        input_element.addEventListener('change', on_change);
    };

    ///
    /// Sets up a control as a popup
    ///
    let load_popup = (popup_node) => {
        // Set the initial state
        let is_open = popup_node.flo_popup_open || false;

        // Function to set whether or not the popup is open or not
        function set_is_open(new_open) {
            is_open = new_open;

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

    return {
        load_slider:    load_slider,
        load_popup:     load_popup,
        layout_popup:   layout_popup
    };
})();
