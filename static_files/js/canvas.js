'use strict';

//  __ _                                 
// / _| |___ ___ __ __ _ _ ___ ____ _ ___
// |  _| / _ \___/ _/ _` | ' \ V / _` (_-<
// |_| |_\___/   \__\__,_|_||_\_/\__,_/__/
//

/* exported flo_canvas */

let flo_canvas = (function() {
    ///
    /// Attaches a new canvas to a HTML element
    ///
    function start(element) {
        let existing_canvas = element.flo_canvas;

        if (!existing_canvas) {
            // Canvas is not present: create a new one
            element.flo_canvas = create_canvas(element);
        } else {
            restart(element, existing_canvas);
        }
    }

    ///
    /// Ensures the canvas element is still part of the item
    ///
    function restart(element, flo_canvas) {
        let parent = flo_canvas.shadow || element;
        let canvas = flo_canvas.canvas;

        // Re-add the canvas if it has no parent node
        if (canvas.parentNode === null) {
            parent.appendChild(canvas);
        }
    }

    ///
    /// Applies a style to canvas
    ///
    function apply_canvas_style(canvas) {
        // Fill the parent node
        canvas.style.width  = '100%';
        canvas.style.height = '100%';

        // Background colour so we can see the canvas
        canvas.style['background-color'] = 'rgba(255,255,255,1.0)';
    }

    ///
    /// Watches a canvas for events
    ///
    function monitor_canvas_events(canvas) {
        let resizing = false;

        ///
        /// Updates the content size of the canvas
        ///
        function resize_canvas() {
            // Queue a resize on the next animation frame
            if (!resizing) {
                resizing = true;
                requestAnimationFrame(() => {
                    resizing = false;

                    // Resize if the canvas's size has changed
                    var ratio           = window.devicePixelRatio || 1;
                    let target_width    = canvas.clientWidth * ratio;
                    let target_height   = canvas.clientHeight * ratio;

                    if (canvas.width !== target_width || canvas.height !== target_height) {
                        // Actually resize the canvas
                        canvas.width    = canvas.clientWidth * ratio;
                        canvas.height   = canvas.clientHeight * ratio;
                    }
                });
            }
        }

        // Register events
        // TODO: as resize events are on the window we'll wind up firing this event even if the canvas element is removed
        window.addEventListener('resize', resize_canvas, false);

        // Run through the initial set of events
        resize_canvas();
    }

    ///
    /// Creates a canvas for an element
    ///
    function create_canvas(element) {
        let shadow = null;
        let parent = element;

        // Put the canvas in the shadow DOM if we can
        if (element.attachShadow) {
            shadow = element.attachShadow({mode: 'open'});
            parent = shadow;
        }

        // Create a new canvas element
        let canvas = document.createElement('canvas');

        // Set up the element
        apply_canvas_style(canvas);
        monitor_canvas_events(canvas);

        // Add it to the DOM
        parent.appendChild(canvas);
        
        // Return the properties to attach to the parent element
        return {
            shadow: shadow,
            canvas: canvas
        };
    }

    // The final flo_canvas object
    return {
        start: start
    };
})();
