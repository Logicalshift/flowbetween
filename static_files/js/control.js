'use strict';

//   __ _                    _           _
//  / _| |___ ___ __ ___ _ _| |_ _ _ ___| |
// |  _| / _ \___/ _/ _ \ ' \  _| '_/ _ \ |
// |_| |_\___/   \__\___/_||_\__|_| \___/_|
//

/* exported flo_control */

let flo_control = (function () {
    // Sets up a control as a slider, with the specified event as an update event
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

    return {
        load_slider: load_slider
    };
})();
