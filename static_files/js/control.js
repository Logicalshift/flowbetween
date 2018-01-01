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

        // Set the input range. We use a fixed range.
        input_element.min = 0.0;
        input_element.max = 1000.0;

        // The 'input' event is fired while the user is changing the slider
        function on_input() {
            // If the node has the range property set, we'll 
            let flo_min = element.flo_min_value || 0.0;
            let flo_max = element.flo_max_value || 100.0;

            // We get a number 0-1000, change to fit in the range
            let value = (input_element.value/1000.0)*(flo_max-flo_min) + flo_min;

            // This is the editing event: if the node has an edit_value handler, this is where we send it
            let input_handler = element.flo_edit_value || (() => {});

            // Pass on the event (sliders generate float values)
            input_handler({ 'Float': value });
        }

        // The 'change' event is fired when the user 
        function on_change() {
            // If the node has the range property set, we'll 
            let flo_min = element.flo_min_value || 0.0;
            let flo_max = element.flo_max_value || 100.0;

            // We get a number 0-1000, change to fit in the range
            let value = (input_element.value/1000.0)*(flo_max-flo_min) + flo_min;

            // This is the set event: if the node has an set_value handler, this is where we send it
            let input_handler = element.flo_set_value || (() => {});

            // Pass on the event (sliders generate float values)
            input_handler({ 'Float': value });
        }

        // Register event handlers
        input_element.addEventListener('input', on_input);
        input_element.addEventListener('change', on_change);
    };

    return {
        load_slider: load_slider
    };
})();
