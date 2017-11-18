'use strict';

//  __ _                                 
// / _| |___ ___ __ __ _ _ ___ ____ _ ___
// |  _| / _ \___/ _/ _` | ' \ V / _` (_-<
// |_| |_\___/   \__\__,_|_||_\_/\__,_/__/
//

/* exported flo_canvas */

let flo_canvas = (function() {
    // List of active canvases
    let active_canvases = [];

    ///
    /// Remove any canvas from the list of active canvases that no longer have a parent
    ///
    function remove_inactive_canvases() {
        // Remove any canvas element that has a null parent
        for (let index=0; index<active_canvases.length; ++index) {
            if (!active_canvases.is_active()) {
                active_canvases.removeAt(index);
                --index;
            }
        }
    }

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
        // Add this canvas to the list of active canvases
        remove_inactive_canvases();
        active_canvases.push({
            is_active:      is_active,
            resize_canvas:  resize_canvas
        });

        ///
        /// Returns true if this canvas is active
        ///
        function is_active() {
            return canvas.parentNode !== null;
        }

        ///
        /// Updates the content size of the canvas
        ///
        function resize_canvas() {
            // Resize if the canvas's size has changed
            var ratio           = window.devicePixelRatio || 1;
            let target_width    = canvas.clientWidth * ratio;
            let target_height   = canvas.clientHeight * ratio;

            if (canvas.width !== target_width || canvas.height !== target_height) {
                // Actually resize the canvas
                canvas.width    = canvas.clientWidth * ratio;
                canvas.height   = canvas.clientHeight * ratio;
            }
        }

        // Run through the initial set of events
        requestAnimationFrame(() => resize_canvas());
    }

    ///
    /// Event callback that resizes any active canvas
    ///
    let resize_active_canvases = (function() {
        let resizing = false;

        return function() {
            // When the resize request comes in, defer it to the next animation frame
            if (!resizing) {
                resizing = true;
                requestAnimationFrame(() => {
                    resizing = false;

                    remove_inactive_canvases();
                    active_canvases.forEach(canvas => {
                        canvas.resize_canvas();
                    });
                });
            }
        };
    })();

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

        // Add it to the DOM
        parent.appendChild(canvas);

        // Set up the element
        apply_canvas_style(canvas);
        monitor_canvas_events(canvas);
        
        // Return the properties to attach to the parent element
        return {
            shadow: shadow,
            canvas: canvas
        };
    }

    // Register global event handlers
    window.addEventListener('resize', resize_active_canvases, false);    

    // The final flo_canvas object
    return {
        start: start
    };
})();
