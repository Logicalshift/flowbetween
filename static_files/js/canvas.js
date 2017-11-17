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
