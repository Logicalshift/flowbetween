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
            if (!active_canvases[index].is_active()) {
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
    /// Creates the drawing functions for a canvas
    ///
    function drawing_functions(canvas) {
        // The replay log will replay the actions that draw this canvas (for example when resizing)
        let replay  = [ clear_canvas ];
        let context = canvas.getContext('2d');

        function new_path()                             { context.beginPath(); }
        function move_to(x,y)                           { context.moveTo(x, y); }
        function line_to(x,y)                           { context.lineTo(x, y); }
        function bezier_curve(x1, y1, x2, y2, x3, y3)   { context.bezierCurveTo(x2, y2, x3, y3, x1, y1); }
        function fill()                                 { context.fill(); }
        function stroke()                               { context.stroke(); }

        function fill_color(r, g, b, a) {
            r = Math.floor(r*255.0);
            g = Math.floor(g*255.0);
            b = Math.floor(b*255.0);

            context.fillStyle = 'rgba(' + r + ',' + g + ',' + b + ',' + a + ')';
        }

        function stroke_color(r, g, b, a) {
            r = Math.floor(r*255.0);
            g = Math.floor(g*255.0);
            b = Math.floor(b*255.0);

            context.strokeStyle = 'rgba(' + r + ',' + g + ',' + b + ',' + a + ')';
        }

        function line_width(width) {
            // TODO: scale to transform?
            context.line_width = width;
        }

        function line_join(join) {
            throw 'Not implemented';
        }

        function line_cap(join) {
            throw 'Not implemented';
        }

        function new_dash_pattern() {
            throw 'Not implemented';
        }

        function dash_length(length) {
            throw 'Not implemented';
        }

        function dash_offset(offset) {
            throw 'Not implemented';
        }

        function blend_mode(blend_mode) {
            throw 'Not implemented';
        }

        function identity_transform() {
            canvas_height(2.0);
        }

        function canvas_height(height) {
            let pixel_width     = canvas.width;
            let pixel_height    = canvas.height;

            let ratio           = pixel_height/height;

            context.setTransform(
                ratio,              0, 
                0,                  -ratio, 
                pixel_width/2.0,    pixel_height/2.0
            );
        }

        function multiply_transform(transform) {
            throw 'Not implemented';
        }

        function unclip() {
            throw 'Not implemented';
        }

        function clip() {
            throw 'Not implemented';
        }

        function store() {
            throw 'Not implemented';
        }

        function restore() {
            throw 'Not implemented';
        }

        function push_state() {
            throw 'Not implemented';
        }

        function pop_state() {
            throw 'Not implemented';
        }

        function clear_canvas() {
            // Clear
            context.resetTransform();
            context.clearRect(0, 0, canvas.width, canvas.height);

            // Reset the transformation and state
            identity_transform();
            fill_color(0,0,0,1);
            stroke_color(0,0,0,1);
            line_width(1.0);
        }

        function replay_drawing() {
            replay.forEach(item => item());
        }

        return {
            new_path:           ()              => { replay.push(new_path);                             new_path();                     },
            move_to:            (x, y)          => { replay.push(() => move_to(x, y));                  move_to(x, y);                  },
            line_to:            (x, y)          => { replay.push(() => line_to(x, y));                  line_to(x, y);                  },
            bezier_curve:       (x1, y1, x2, y2, x3, y3) => { replay.push(() => bezier_curve(x1, y1, x2, y2, x3, y3)); bezier_curve(x1, y1, x2, x2, y3, y3); },
            fill:               ()              => { replay.push(fill);                                 fill();                         },
            stroke:             ()              => { replay.push(stroke);                               stroke();                       },
            line_width:         (width)         => { replay.push(() => line_width(width));              line_width(width);              },
            line_join:          (join)          => { replay.push(() => line_join(join));                line_join(join);                },
            line_cap:           (cap)           => { replay.push(() => line_cap(cap));                  line_cap(cap);                  },
            new_dash_pattern:   ()              => { replay.push(new_dash_pattern);                     new_dash_pattern();             },
            dash_length:        (length)        => { replay.push(() => dash_length(length));            dash_length(length);            },
            dash_offset:        (offset)        => { replay.push(() => dash_offset(offset));            dash_length(offset);            },
            fill_color:         (r, g, b, a)    => { replay.push(() => fill_color(r, g, b, a));         fill_color(r, g, b, a);         },
            stroke_color:       (r, g, b, a)    => { replay.push(() => stroke_color(r, g, b, a));       stroke_color(r, g, b, a);       },
            blend_mode:         (mode)          => { replay.push(() => blend_mode(mode));                blend_mode(mode);              },
            identity_transform: ()              => { replay.push(identity_transform);                   identity_transform();           },
            canvas_height:      (height)        => { replay.push(() => canvas_height(height));          canvas_height(height);          },
            multiply_transform: (transform)     => { replay.push(() => multiply_transform(transform));  multiply_transform(transform);  },
            unclip:             ()              => { replay.push(unclip);                               unclip();                       },
            clip:               ()              => { replay.push(clip);                                 clip();                         },
            store:              ()              => { replay.push(store);                                store();                        },
            restore:            ()              => { replay.push(restore);                              restore();                      },
            push_state:         ()              => { replay.push(push_state);                           push_state();                   },
            pop_state:          ()              => { replay.push(pop_state);                            pop_state();                    },
            clear_canvas:       ()              => { replay = [ clear_canvas ];                         clear_canvas();                 },

            replay_drawing:     replay_drawing
        };
    }

    ///
    /// Applies a style to canvas
    ///
    function apply_canvas_style(canvas) {
        // Fill the parent node
        canvas.style.width  = '100%';
        canvas.style.height = '100%';
    }

    ///
    /// Watches a canvas for events
    ///
    function monitor_canvas_events(canvas) {
        // Add this canvas to the list of active canvases
        remove_inactive_canvases();
        active_canvases.push({
            is_active:      is_active,
            resize_canvas:  resize_canvas,
            draw:           canvas.flo_draw
        });

        let draw = canvas.flo_draw;

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

                // Redraw the canvas contents at the new size
                draw.replay_drawing();
            }
        }

        // Run through the initial set of events
        requestAnimationFrame(() => resize_canvas());
    }

    ///
    /// Causes all of the canvases to adjust their size (can be hooked up to the
    /// window resize event to ensure that all canvases are the right size)
    ///
    function resize_active_canvases() {
        remove_inactive_canvases();
        active_canvases.forEach(canvas => {
            canvas.resize_canvas();
        });
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

        // Add it to the DOM
        parent.appendChild(canvas);

        // Set up the element
        let draw            = drawing_functions(canvas);
        element.flo_draw    = draw;
        canvas.flo_draw     = draw;

        // Test drawing
        draw.clear_canvas();
        draw.fill_color(1, 0, 0, 0.5);
        draw.move_to(-0.5,-0.5);
        draw.line_to(-0.5, 0.5);
        draw.line_to(1.0, 1.0);
        draw.line_to(0.5, -0.5);
        draw.line_to(-0.5,-0.5);
        draw.fill();

        apply_canvas_style(canvas);
        monitor_canvas_events(canvas);
        
        // Return the properties to attach to the parent element
        return {
            shadow: shadow,
            canvas: canvas,
            draw:   canvas.flo_draw
        };
    }

    // The final flo_canvas object
    return {
        start:              start,
        resize_canvases:    resize_active_canvases
    };
})();
