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
        function on_input(event) {
            console.log('Input', event);
        }

        // The 'change' event is fired when the user 
        function on_change(event) {
            console.log('Change', event);
        }

        // Register event handlers
        input_element.addEventListener('input', on_input);
        input_element.addEventListener('change', on_change);
    };

    return {
        load_slider: load_slider
    };
})();
