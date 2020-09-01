'use strict';

//   __ _                                 
//  / _| |___ ___ __ __ _ _ ___ ____ _ ___
// |  _| / _ \___/ _/ _` | ' \ V / _` (_-<
// |_| |_\___/   \__\__,_|_||_\_/\__,_/__/
//

/* global flo_matrix */
/* exported flo_canvas */

let flo_canvas = (function() {
    // List of active canvases
    let active_canvases = [];

    // Canvases in the 'boneyard', which can be resurrected if they're used again quickly
    let boneyard        = [];

    // Maps controller_path + canvas_name to the list of active canvases
    let canvas_map      = {};

    // True if the canvas map is outdated
    let canvas_map_outdated = true;

    ///
    /// Removes dead canvases from the boneyard
    ///
    function reap_boneyard() {
        boneyard = [];
    }

    ///
    /// Remove any canvas from the list of active canvases that no longer have a parent
    ///
    function remove_inactive_canvases() {
        let new_active_canvases = [];

        // Remove any canvas element that has a null parent
        for (let index=0; index<active_canvases.length; ++index) {
            if (active_canvases[index].is_active()) {
                new_active_canvases.push(active_canvases[index]);
            } else {
                boneyard.push(active_canvases[index]);
                mark_canvases_outdated();
            }
        }

        active_canvases = new_active_canvases;
    }

    ///
    /// Updates the canvas map from the list of active canvases
    ///
    function update_canvas_map() {
        // Ensure that only active canvases are considered
        remove_inactive_canvases();

        // Generate a new canvas map from the list of active canvases
        let new_canvas_map = {};

        active_canvases.forEach(canvas => {
            let address = canvas.flo_controller + '/' + canvas.flo_name;
            new_canvas_map[address] = canvas;
        });
        
        // Store as the canvas map
        canvas_map          = new_canvas_map;
        canvas_map_outdated = false;
    }

    ///
    /// Indicates that the canvases are outdated
    ///
    function mark_canvases_outdated() {
        if (!canvas_map_outdated) {
            canvas_map_outdated = true;
            requestAnimationFrame(() => {
                if (canvas_map_outdated) {
                    update_canvas_map();
                    reap_boneyard();
                }
            });
        }
    }

    ///
    /// Attaches a new canvas to a HTML element
    ///
    function start(element) {
        remove_inactive_canvases();

        let existing_canvas = element.flo_canvas;

        if (!existing_canvas) {
            // Attempt to raise the canvas from the dead
            let flo_controller  = element.getAttribute('flo-controller');
            let flo_name        = element.getAttribute('flo-name');
            let zombie          = get_zombie_canvas(flo_controller, flo_name);

            if (zombie) {
                // Can resurrect the canvas using an existing element
                element.flo_canvas = resurrect_canvas(element, zombie);
            } else {
                // Need to create an all-new canvas
                element.flo_canvas = create_canvas(element);
            }
        } else {
            // Canvas is already set up, but isn't started
            restart(element, existing_canvas);
        }
    }

    ///
    /// Detaches a canvas from an HTML element
    ///
    function stop(element) {
        let existing_canvas = element.flo_canvas;

        if (existing_canvas) {
            // Remove the existing canvas from the element
            element.flo_canvas = null;
            existing_canvas.canvas.remove();

            // Make sure we know that the canvases are outdated
            mark_canvases_outdated();
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
    function create_drawing_functions(canvas) {
        let context                     = canvas.getContext('2d');
        let current_path                = [];
        let context_stack               = [];
        let clip_stack                  = [];
        let clipped                     = false;
        let transform                   = [1,0,0, 0,1,0, 0,0,1];
        let inverse_transform           = null;
        let dash_pattern                = [];
        let set_dash_pattern            = true;
        let stored_pixels               = document.createElement('canvas');
        let generate_buffer_on_store    = false;
        let have_stored_image           = false;
        let last_store_pos              = null;
        let layer_canvases              = null;
        let blend_for_layer             = {};
        let current_layer_id            = 0;
        let current_sprite              = [ ];
        let sprites                     = { };
        let sprite_transform            = [1,0,0, 0,1,0, 0,0,1];

        ///
        /// Sets the current transform (lack of browser support for currentTransform means we have to track this independently)
        ///
        function transform_set(new_transform) {
            transform           = new_transform;
            inverse_transform   = null;
        }
        
        ///
        /// Multiplies the transformation matrix (lack of browser support again)
        ///
        function transform_multiply(new_transform) {
            let t1 = transform;
            let t2 = new_transform;

            let res = [
                t1[0]*t2[0] + t1[1]*t2[3] + t1[2]*t2[6],
                t1[0]*t2[1] + t1[1]*t2[4] + t1[2]*t2[7],
                t1[0]*t2[2] + t1[1]*t2[5] + t1[2]*t2[8],

                t1[3]*t2[0] + t1[4]*t2[3] + t1[5]*t2[6],
                t1[3]*t2[1] + t1[4]*t2[4] + t1[5]*t2[7],
                t1[3]*t2[2] + t1[4]*t2[5] + t1[5]*t2[8],

                t1[6]*t2[0] + t1[7]*t2[3] + t1[8]*t2[6],
                t1[6]*t2[1] + t1[7]*t2[4] + t1[8]*t2[7],
                t1[6]*t2[2] + t1[7]*t2[5] + t1[8]*t2[8],
            ];

            transform           = res;
            inverse_transform   = null;
        }

        ///
        /// Multiplies the sprite transformation matrix
        ///
        function sprite_transform_multiply(new_transform) {
            let t1 = sprite_transform;
            let t2 = new_transform;

            let res = [
                t1[0]*t2[0] + t1[1]*t2[3] + t1[2]*t2[6],
                t1[0]*t2[1] + t1[1]*t2[4] + t1[2]*t2[7],
                t1[0]*t2[2] + t1[1]*t2[5] + t1[2]*t2[8],

                t1[3]*t2[0] + t1[4]*t2[3] + t1[5]*t2[6],
                t1[3]*t2[1] + t1[4]*t2[4] + t1[5]*t2[7],
                t1[3]*t2[2] + t1[4]*t2[5] + t1[5]*t2[8],

                t1[6]*t2[0] + t1[7]*t2[3] + t1[8]*t2[6],
                t1[6]*t2[1] + t1[7]*t2[4] + t1[8]*t2[7],
                t1[6]*t2[2] + t1[7]*t2[5] + t1[8]*t2[8],
            ];

            sprite_transform    = res;
            inverse_transform   = null;
        }

        ///
        /// Copies the contents of one canvas to another one
        ///
        function copy_canvas(src_canvas, target_canvas) {
            let target_context = target_canvas.getContext('2d');

            target_context.save();
            target_context.setTransform(1,0, 0,1, 0,0);
            target_context.globalCompositeOperation = 'copy';

            target_context.drawImage(src_canvas, 0,0, src_canvas.width, src_canvas.height);

            target_context.restore();
        }

        ///
        /// Removes the clipping path if one is applied
        ///
        function remove_clip() {
            // TODO: because JS isn't very well designed, this will clobber things like 
            // the fill style as well which we don't want to happen.
            // (The design issues here are a combination of context.save saving
            // absolutely everything and there being no way to remove a clipping
            // path once applied)
            if (clipped) {
                clipped = false;
                context.restore();
            }
        }

        ///
        /// Restores the clipping path if it's missing
        ///
        function restore_clip() {
            if (!clipped && clip_stack.length > 0) {
                clipped = true;
                context.save();
                clip_stack.forEach(fn => fn());
            }
        }

        ///
        /// Creates a new layer
        ///
        function create_layer() {
            let new_layer       = document.createElement('canvas');
            new_layer.width     = canvas.width;
            new_layer.height    = canvas.height;

            return new_layer;
        }

        // Specify the rendering functions (there are two sets: the layer renderers write straight to canvases, and the sprite renderer records rendering actions for later replay)
        let render          = null;
        let layer_renderer  = null;
        let sprite_renderer = null;

        layer_renderer = {
            new_path: ()                             => { context.beginPath(); current_path = []; },
            move_to: (x,y)                           => { context.moveTo(x, y); current_path.push(() => context.moveTo(x, y) ); },
            line_to: (x,y)                           => { context.lineTo(x, y); current_path.push(() => context.lineTo(x, y) ); },
            bezier_curve: (x1, y1, x2, y2, x3, y3)   => { context.bezierCurveTo(x2, y2, x3, y3, x1, y1); current_path.push(() => context.bezierCurveTo(x2, y2, x3, y3, x1, y1) ); },
            close_path: ()                           => { context.closePath(); },
            fill: ()                                 => { context.fill(); },

            stroke: () => {
                if (set_dash_pattern) {
                    set_dash_pattern = false;
                    context.setLineDash(dash_pattern);
                }

                context.stroke(); 
            },

            fill_color: (r, g, b, a) => {
                r = Math.floor(r*255.0);
                g = Math.floor(g*255.0);
                b = Math.floor(b*255.0);

                context.fillStyle = 'rgba(' + r + ',' + g + ',' + b + ',' + a + ')';
            },

            stroke_color: (r, g, b, a) => {
                r = Math.floor(r*255.0);
                g = Math.floor(g*255.0);
                b = Math.floor(b*255.0);

                context.strokeStyle = 'rgba(' + r + ',' + g + ',' + b + ',' + a + ')';
            },

            line_width: (width) => {
                context.lineWidth = width;
            },

            line_width_pixels: (width) => {
                // Length of the first column of the transformation matrix is the scale factor (for the width)
                let scale = Math.sqrt(transform[0]*transform[0] + transform[3]*transform[3]);
                if (scale === 0) scale = 1;
                //scale /= window.devicePixelRatio || 1;

                // Scale the width down according to this factor (we'll always use the horizontal scale factor)
                context.lineWidth = width / scale;
            },

            line_join: (join) => {
                context.lineJoin = join;
            },

            line_cap: (cap) => {
                context.lineCap = cap;
            },

            new_dash_pattern: () => {
                dash_pattern        = [];
                set_dash_pattern    = true;
            },

            dash_length: (length) => {
                context.lineDashOffset = length;
            },

            dash_offset: (offset) => {
                dash_pattern.push(offset);
                set_dash_pattern = true;
            },

            blend_mode: (blend_mode) => {
                context.globalCompositeOperation = blend_mode;
            },

            identity_transform: () => {
                canvas_height(2.0);
            },

            canvas_height: (height) => {
                let pixel_width     = canvas.width;
                let pixel_height    = canvas.height;

                let ratio_x         = pixel_height/height;
                let ratio_y         = -ratio_x;

                if (height < 0) {
                    // Using a negative heights flips coordinates vertically but not horizontally
                    ratio_x = -ratio_x;
                }

                context.setTransform(
                    ratio_x,            0, 
                    0,                  ratio_y, 
                    pixel_width/2.0,    pixel_height/2.0
                );

                transform_set([ 
                    ratio_x,    0,          pixel_width/2.0,
                    0,          ratio_y,    pixel_height/2.0, 
                    0,          0,          1
                ]);
            },

            center_region: (minx, miny, maxx, maxy) => {
                let pixel_width     = canvas.width;
                let pixel_height    = canvas.height;

                // Get the current scaling of this canvas
                let xscale = Math.sqrt(transform[0]*transform[0] + transform[3]*transform[3]);
                let yscale = Math.sqrt(transform[1]*transform[1] + transform[4]*transform[4]);
                if (xscale === 0) xscale = 1;
                if (yscale === 0) yscale = 1;

                // Current X, Y coordinates (centered)
                let cur_x = (transform[2]-(pixel_width/2.0))/xscale;
                let cur_y = (transform[5]-(pixel_height/2.0))/yscale;
                
                // New center coordinates
                let center_x = (minx+maxx)/2.0;
                let center_y = (miny+maxy)/2.0;

                // Compute the offsets and transform the canvas
                let x_offset = cur_x - center_x;
                let y_offset = cur_y - center_y;

                layer_renderer.multiply_transform([
                    1, 0, x_offset,
                    0, 1, y_offset,
                    0, 0, 1
                ]);
            },

            multiply_transform: (transform) => {
                // Rotated transformation matrix
                context.transform(transform[0], transform[3], transform[1], transform[4], transform[2], transform[5]);
                transform_multiply(transform);
            },

            unclip: () => {
                // Stop clipping and clear the stack
                remove_clip();
                clip_stack = [];
            },

            clip: () => {
                // Make sure the clipping path is turned on
                restore_clip();

                // Need to be able to restore the clipping path
                let clip_path = current_path.slice();
                clip_stack.push(() => {
                    clip_path.forEach(fn => fn());
                    context.clip();
                });

                // Add the current path to the context
                context.clip();
            },

            store: () => {
                if (generate_buffer_on_store) {
                    stored_pixels = document.createElement('canvas');
                }

                // Update the size of the backing buffer
                let width       = canvas.width;
                let height      = canvas.height;

                if (width !== stored_pixels.width)      { stored_pixels.width = canvas.width; }
                if (height !== stored_pixels.height)    { stored_pixels.height = canvas.height; }

                let source_canvas = canvas;
                if (layer_canvases) {
                    source_canvas = layer_canvases[current_layer_id];
                }
                
                // Remember where the store was in the replay (so we can rewind)
                last_store_pos  = replay.length;

                // Draw the canvas to the backing buffer (we use a backing canvas because getImageData is very slow on all browsers)
                let stored_context = stored_pixels.getContext('2d');
                stored_context.globalCompositeOperation = 'copy';
                stored_context.drawImage(source_canvas, 0, 0);
                have_stored_image = true;
            },

            restore: () => {
                // Reset the image data to how it was at the last point it was used
                if (have_stored_image) {
                    context.save();

                    context.globalCompositeOperation = 'copy';
                    context.setTransform(
                        1,  0, 
                        0,  1, 
                        0,  0
                    );
                    context.drawImage(stored_pixels, 0, 0);

                    context.restore();
                }
            },

            free_stored_buffer: () => {
                // Set that we no longer have a stored image
                if (have_stored_image) {
                    have_stored_image   = false;
                }
            },

            push_state: () => {
                // Push the current clipping path and dash pattern
                let restore_clip_stack          = clip_stack.slice();
                let restore_dash_pattern        = dash_pattern.slice();
                let restore_stored_pixels       = stored_pixels;
                let restore_gen_buffer          = generate_buffer_on_store;
                let restore_have_image          = have_stored_image;
                let restore_layer_id            = current_layer_id;
                let restore_last_store_pos      = last_store_pos;
                let restore_transform           = transform.slice();
                let restore_sprite_transform    = sprite_transform.slice();
                context_stack.push(() => {
                    clip_stack                  = restore_clip_stack;
                    dash_pattern                = restore_dash_pattern;
                    stored_pixels               = restore_stored_pixels;
                    generate_buffer_on_store    = restore_gen_buffer;
                    have_stored_image           = restore_have_image;
                    current_layer_id            = restore_layer_id;
                    last_store_pos              = restore_last_store_pos;
                    transform                   = restore_transform;
                    sprite_transform            = restore_sprite_transform;
                    set_dash_pattern            = true;
                });

                // Cannot rewind the replay if we restore pixels pushed before this state (while the state is in effect)
                last_store_pos = null;
                
                // If we store a new buffered image while a state is pushed, then we need a new canvas to store it in
                if (have_stored_image) {
                    generate_buffer_on_store = true;
                }

                // Save the context with no clipping path (so we can unclip)
                remove_clip();
                context.save();
                restore_clip();
            },

            pop_state: () => {
                if (context_stack.length === 0) {
                    console.warn('Tried to pop state while stack was empty');
                    return;
                }

                // Remove any clipping we have
                remove_clip();

                // Restore state not saved in context
                context_stack.pop()();

                // Restore context state
                context.restore();

                // Reinstate the clipping
                restore_clip();
            },

            layer: (layer_id) => {
                // Clear any existing clipping rect
                unclip();

                // Switch to layer rendering
                render = layer_renderer;

                // Set up layers if none are defined
                if (!layer_canvases) {
                    layer_canvases = {};

                    // Create the initial layer
                    layer_canvases[0] = create_layer();
                    copy_canvas(canvas, layer_canvases[0]);

                    // Clear the main context
                    context.setTransform(1,0, 0,1, 0,0);
                    context.resetTransform();
                    context.clearRect(0, 0, canvas.width, canvas.height);
                }

                // Create a new layer if this ID doesn't exist
                let existing_layer = layer_canvases[layer_id];
                if (!existing_layer) {
                    existing_layer = layer_canvases[layer_id] = create_layer();
                }

                // Set the context to this layer
                context             = existing_layer.getContext('2d');
                current_layer_id    = layer_id;

                // Copy the transform to this layer
                context.setTransform(
                    transform[0],transform[3], 
                    transform[1],transform[4], 
                    transform[2],transform[5]
                );
            },

            clear_layer: () => {
                // Clear the current layer
                context.resetTransform();
                context.clearRect(0, 0, canvas.width, canvas.height);
                context.setTransform(
                    transform[0],transform[3], 
                    transform[1],transform[4], 
                    transform[2],transform[5]
                );

                // Reset the blend mode
                blend_for_layer[current_layer_id] = 'source-over';

                // Remove everything from the canvas that was on this layer
                for (let index=1; index<replay.length; ++index) {
                    if (replay[index][2] === current_layer_id) {
                        replay[index] = null;
                        replay.splice(index, 1);
                        --index;
                    }
                }

                // Add a 'set layer' command to the replay
                replay.push([layer, [current_layer_id], current_layer_id]);
            },

            layer_blend: (layer_id, blend_mode) => {
                blend_for_layer[layer_id] = blend_mode;
            },

            clear_canvas: () => {
                // Clear layers
                layer_canvases      = null;
                context             = canvas.getContext('2d');
                blend_for_layer     = {};
                current_layer_id    = 0;
                render              = layer_renderer;

                // Clear
                context.setTransform(1,0, 0,1, 0,0);
                context.resetTransform();
                context.clearRect(0, 0, canvas.width, canvas.height);

                // Reset the transformation and state
                current_path    = [];
                clip_stack      = [];

                stored_pixels.width  = 1;
                stored_pixels.height = 1;

                identity_transform();
                fill_color(0,0,0,1);
                stroke_color(0,0,0,1);
                line_width(1.0);
            },

            sprite: (sprite_id) => {
                // Switch to sprite rendering
                render = sprite_renderer;

                // Select the specified sprite
                if (!sprites[sprite_id]) {
                    sprites[sprite_id] = [ ];
                }

                current_sprite = sprites[sprite_id];
            },

            clear_sprite: () => {
            },

            draw_sprite: (sprite_id) => {
                let sprite = sprites[sprite_id];
                if (sprite) {
                    // Push the state before the sprite
                    layer_renderer.push_state();

                    // Apply the sprite transform
                    layer_renderer.multiply_transform(sprite_transform);

                    // Run the sprite commands
                    sprite.forEach(item => item[0].apply(null, item[1]));

                    // Restore the state as it was before the sprite
                    layer_renderer.pop_state();
                }
            },

            sprite_transform_identity: () => {
                sprite_transform = [1,0,0, 0,1,0, 0,0,1];
            },

            sprite_transform_translate: (x, y) => {
                sprite_transform_multiply([
                    1, 0, x,
                    0, 1, y,
                    0, 0, 1
                ]);
            },

            sprite_transform_scale: (x, y) => {
                sprite_transform_multiply([
                    x, 0, 0,
                    0, y, 0,
                    0, 0, 1
                ]);

            },

            sprite_transform_rotate: (angle) => {
                let radians = angle / 180.0 * Math.PI;
                let cos     = Math.cos(radians);
                let sin     = Math.sin(radians);

                sprite_transform_multiply([
                    [cos,   -sin,   0.0],
                    [sin,   cos,    0.0],
                    [0.0,   0.0,    1.0]
                ]);
            },

            sprite_transform_matrix: (matrix) => {
                sprite_transform_multiply(matrix);
            }
        };

        function rewind_to_last_store() {
            if (last_store_pos !== null) {
                while (replay.length > last_store_pos) {
                    replay.pop();
                }
            }
        }

        function rewind_free_stored() {
            // If the top of the replay buffer is 'store, free stored' remove them both
            if (replay.length >= 2) {
                let free_stored_index   = replay.length-1;
                let store_index         = replay.length-2;

                if (replay[free_stored_index][0] === free_stored_buffer && replay[store_index][0] === store) {
                    replay.pop();
                    replay.pop();
                }
            }
        }

        function replay_drawing() {
            replay.forEach(item => item[0].apply(null, item[1]));
        }

        function map_coords(x, y) {
            // Invert the active transformation matrix if it's not already inverted
            if (inverse_transform === null) {
                inverse_transform = flo_matrix.invert3(transform);
            }

            // Assuming square pixels, map x,y to internal canvas coords
            let ratio = canvas.width / canvas.clientWidth;

            // Use the inverse matrix to map the coordinates
            return flo_matrix.mulvec3(inverse_transform, [x*ratio, y*ratio, 1]);
        }

        function draw_layers() {
            // If we're using layers, then this must be called to update the canvas (if layers are not in use, it'll update directly)
            // (This is a bit awkward if we're updating the canvas manually: we want to avoid calling this too often, though)
            if (layer_canvases) {
                // Draw on the main canvas
                let layer_context = canvas.getContext('2d');
                layer_context.setTransform(1,0, 0,1, 0,0);

                let width   = canvas.width;
                let height  = canvas.height;

                // Clear the canvas
                layer_context.clearRect(0, 0, width, height);

                // Draw each of the layers
                Object.keys(layer_canvases).forEach(layer_id => {
                    layer_context.globalCompositeOperation = blend_for_layer[layer_id] || 'source-over';
                    layer_context.drawImage(layer_canvases[layer_id], 0,0, width,height);
                });
            }
        }

        // The sprite renderer just records actions in the current sprite
        sprite_renderer = { 
            new_path:                       ()                          => { current_sprite.push([new_path, []]); },
            move_to:                        (x, y)                      => { current_sprite.push([move_to, [x, y]]); },
            line_to:                        (x, y)                      => { current_sprite.push([line_to, [x, y]]); },
            bezier_curve:                   (x1, y1, x2, y2, x3, y3)    => { current_sprite.push([bezier_curve, [x1, y1, x2, y2, x3, y3]]); },
            close_path:                     ()                          => { current_sprite.push([close_path, []]); },
            fill:                           ()                          => { current_sprite.push([fill, []]); },
            stroke:                         ()                          => { current_sprite.push([stroke, []]); },
            line_width:                     (width)                     => { current_sprite.push([line_width, [width]]); },
            line_width_pixels:              (width)                     => { current_sprite.push([line_width_pixels, [width]]); },
            line_join:                      (join)                      => { current_sprite.push([line_join, [join]]); },
            line_cap:                       (cap)                       => { current_sprite.push([line_cap, [cap]]); },
            new_dash_pattern:               ()                          => { current_sprite.push([new_dash_pattern, []]); },
            dash_length:                    (length)                    => { current_sprite.push([dash_length, [length]]); },
            dash_offset:                    (offset)                    => { current_sprite.push([dash_offset, [offset]]); },
            fill_color:                     (r, g, b, a)                => { current_sprite.push([fill_color, [r, g, b, a]]); },
            stroke_color:                   (r, g, b, a)                => { current_sprite.push([stroke_color, [r, g, b, a]]); },
            blend_mode:                     (mode)                      => { current_sprite.push([blend_mode, [mode]]); },
            identity_transform:             ()                          => { current_sprite.push([identity_transform, []]); },
            canvas_height:                  (height)                    => { current_sprite.push([canvas_height, [height]]); },
            center_region:                  (x1, y1, x2, y2)            => { current_sprite.push([center_region, [x1, y1, x2, y2]]); },
            multiply_transform:             (transform)                 => { current_sprite.push([multiply_transform, [transform]]); },
            unclip:                         ()                          => { current_sprite.push([unclip, []]); },
            clip:                           ()                          => { current_sprite.push([clip, []]); },
            store:                          ()                          => { current_sprite.push([store, []]); },
            restore:                        ()                          => { current_sprite.push([restore, []]); },
            free_stored_buffer:             ()                          => { current_sprite.push([free_stored_buffer, []]); },
            push_state:                     ()                          => { current_sprite.push([push_state, []]); },
            pop_state:                      ()                          => { current_sprite.push([pop_state, []]); },
            layer_blend:                    (blend_mode)                => { current_sprite.push([layer_blend, [blend_mode]]); },
            clear_layer:                    ()                          => { current_sprite.push([clear_layer, []]); },
            clear_canvas:                   ()                          => { current_sprite.push([clear_canvas, []]); },
            draw_sprite:                    (sprite_id)                 => { current_sprite.push([draw_sprite, [sprite_id]]); },
            sprite_transform_identity:      ()                          => { current_sprite.push([sprite_transform_identity, []]); },
            sprite_transform_translate:     (x, y)                      => { current_sprite.push([sprite_transform_translate, [x, y]]); },
            sprite_transform_scale:         (x, y)                      => { current_sprite.push([sprite_transform_scale, [x, y]]); },
            sprite_transform_rotate:        (angle)                     => { current_sprite.push([sprite_transform_rotate, [angle]]); },
            sprite_transform_matrix:        (matrix)                    => { current_sprite.push([sprite_transform_matrix, [matrix]]); },

            layer:                          (layer_id)                  => { layer_renderer.layer(layer_id); },
            sprite:                         (sprite_id)                 => { layer_renderer.sprite(sprite_id); },
            clear_sprite:                   ()                          => { current_sprite.length = 0; },
        };

        // Render points to the active set of rendering functions
        render      = layer_renderer;

        // This set of functions encapsulate the render state and are used when replaying actions
        function new_path()                             { render.new_path(); }
        function move_to(x, y)                          { render.move_to(x, y); }
        function line_to(x, y)                          { render.line_to(x, y); }
        function bezier_curve(x1, y1, x2, y2, x3, y3)   { render.bezier_curve(x1, y1, x2, y2, x3, y3); }
        function close_path()                           { render.close_path(); }
        function fill()                                 { render.fill(); }
        function stroke()                               { render.stroke(); }
        function line_width(width)                      { render.line_width(width); }
        function line_width_pixels(width)               { render.line_width_pixels(width); }
        function line_join(join)                        { render.line_join(join); }
        function line_cap(cap)                          { render.line_cap(cap); }
        function new_dash_pattern()                     { render.new_dash_pattern(); }
        function dash_length(length)                    { render.dash_length(length); }
        function dash_offset(offset)                    { render.dash_offset(offset); }
        function fill_color(r, g, b, a)                 { render.fill_color(r, g, b, a); }
        function stroke_color(r, g, b, a)               { render.stroke_color(r, g, b, a); }
        function blend_mode(mode)                       { render.blend_mode(mode); }
        function identity_transform()                   { render.identity_transform(); }
        function canvas_height(height)                  { render.canvas_height(height); }
        function center_region(x1, y1, x2, y2)          { render.center_region(x1, y1, x2, y2); }
        function multiply_transform(transform)          { render.multiply_transform(transform); }
        function unclip()                               { render.unclip(); }
        function clip()                                 { render.clip(); }
        function store()                                { render.store(); }
        function restore()                              { render.restore(); }
        function free_stored_buffer()                   { render.free_stored_buffer(); }
        function push_state()                           { render.push_state(); }
        function pop_state()                            { render.pop_state(); }
        function layer(layer_id)                        { render.layer(layer_id); }
        function layer_blend(blend_mode)                { render.layer_blend(blend_mode); }
        function clear_layer()                          { render.clear_layer(); }
        function clear_canvas()                         { render.clear_canvas(); }
        function sprite(sprite_id)                      { render.sprite(sprite_id); }
        function clear_sprite()                         { render.clear_sprite(); }
        function draw_sprite(sprite_id)                 { render.draw_sprite(sprite_id); }
        function sprite_transform_identity()            { render.sprite_transform_identity(); }
        function sprite_transform_translate(x, y)       { render.sprite_transform_translate(x, y); }
        function sprite_transform_scale(x, y)           { render.sprite_transform_scale(x, y); }
        function sprite_transform_rotate(angle)         { render.sprite_transform_rotate(angle); }
        function sprite_transform_matrix(matrix)        { render.sprite_transform_matrix(matrix); }        

        // The replay log will replay the actions that draw this canvas (for example when resizing)
        let replay  = [ [ clear_canvas, [] ] ];

        return {
            new_path:           ()              => { replay.push([new_path, [], current_layer_id]);                         render.new_path();                     },
            move_to:            (x, y)          => { replay.push([move_to, [x, y], current_layer_id]);                      render.move_to(x, y);                  },
            line_to:            (x, y)          => { replay.push([line_to, [x, y], current_layer_id]);                      render.line_to(x, y);                  },
            bezier_curve:       (x1, y1, x2, y2, x3, y3) => { replay.push([bezier_curve, [x1, y1, x2, y2, x3, y3], current_layer_id]); render.bezier_curve(x1, y1, x2, y2, x3, y3); },
            close_path:         ()              => { replay.push([close_path, [], current_layer_id]);                       render.close_path();                   },
            fill:               ()              => { replay.push([fill, [], current_layer_id]);                             render.fill();                         },
            stroke:             ()              => { replay.push([stroke, [], current_layer_id]);                           render.stroke();                       },
            line_width:         (width)         => { replay.push([line_width, [width], current_layer_id]);                  render.line_width(width);              },
            line_width_pixels:  (width)         => { replay.push([line_width_pixels, [width], current_layer_id]);           render.line_width_pixels(width);       },
            line_join:          (join)          => { replay.push([line_join, [join], current_layer_id]);                    render.line_join(join);                },
            line_cap:           (cap)           => { replay.push([line_cap, [cap], current_layer_id]);                      render.line_cap(cap);                  },
            new_dash_pattern:   ()              => { replay.push([new_dash_pattern, [], current_layer_id]);                 render.new_dash_pattern();             },
            dash_length:        (length)        => { replay.push([dash_length, [length], current_layer_id]);                render.dash_length(length);            },
            dash_offset:        (offset)        => { replay.push([dash_offset, [offset], current_layer_id]);                render.dash_length(offset);            },
            fill_color:         (r, g, b, a)    => { replay.push([fill_color, [r, g, b, a], current_layer_id]);             render.fill_color(r, g, b, a);         },
            stroke_color:       (r, g, b, a)    => { replay.push([stroke_color, [r, g, b, a], current_layer_id]);           render.stroke_color(r, g, b, a);       },
            blend_mode:         (mode)          => { replay.push([blend_mode, [mode], current_layer_id]);                   render.blend_mode(mode);               },
            identity_transform: ()              => { replay.push([identity_transform, [], current_layer_id]);               render.identity_transform();           },
            canvas_height:      (height)        => { replay.push([canvas_height, [height], current_layer_id]);              render.canvas_height(height);          },
            center_region:      (x1, y1, x2, y2) => { replay.push([center_region, [x1, y1, x2, y2], current_layer_id]);     render.center_region(x1, y1, x2, y2);  },
            multiply_transform: (transform)     => { replay.push([multiply_transform, [transform], current_layer_id]);      render.multiply_transform(transform);  },
            unclip:             ()              => { replay.push([unclip, [], current_layer_id]);                           render.unclip();                       },
            clip:               ()              => { replay.push([clip, [], current_layer_id]);                             render.clip();                         },
            store:              ()              => { replay.push([store, [], current_layer_id]);                            render.store();                        },
            restore:            ()              => { replay.push([restore, [], current_layer_id]); rewind_to_last_store();  render.restore();                      },
            free_stored_buffer: ()              => { replay.push([free_stored_buffer, [], current_layer_id]); rewind_free_stored(); render.free_stored_buffer();   },
            push_state:         ()              => { replay.push([push_state, [], current_layer_id]);                       render.push_state();                   },
            pop_state:          ()              => { replay.push([pop_state, [], current_layer_id]);                        render.pop_state();                    },
            layer:              (layer_id)      => { replay.push([layer, [layer_id], layer]);                               render.layer(layer_id);                },
            layer_blend:        (layer_id, blend_mode) => { replay.push([layer_blend, [layer_id, blend_mode], -1]);         render.layer_blend(layer_id, blend_mode); },
            clear_layer:        ()              => { replay.push([clear_layer, [], current_layer_id]);                      render.clear_layer();                  },
            clear_canvas:       ()              => { replay = [ [clear_canvas, [], current_layer_id] ];                     render.clear_canvas();                 },
            sprite:             (sprite_id)     => { replay = [ [sprite, [sprite_id], current_layer_id] ];                  render.sprite(sprite_id);              },
            clear_sprite:       ()              => { replay = [ [clear_sprite, [], current_layer_id] ];                     render.clear_sprite();                 },
            draw_sprite:        (sprite_id)     => { replay = [ [draw_sprite, [sprite_id], current_layer_id] ];             render.draw_sprite(sprite_id);         },

            sprite_transform_identity:  ()          => { replay = [ [sprite_transform_identity, [], current_layer_id] ];        render.sprite_transform_identity();        },
            sprite_transform_translate: (x, y)      => { replay = [ [sprite_transform_translate, [x, y], current_layer_id ] ];  render.sprite_transform_translate(x, y);   },
            sprite_transform_scale:     (x, y)      => { replay = [ [sprite_transform_scale, [x, y], current_layer_id] ];       render.sprite_transform_scale(x, y);       },
            sprite_transform_rotate:    (angle)     => { replay = [ [sprite_transform_rotate, [angle], current_layer_id] ];     render.sprite_transform_rotate(angle);     },
            sprite_transform_matrix:    (matrix)    => { replay = [ [sprite_transform_matrix, [matrix], current_layer_id] ];    render.sprite_transform_matrix(matrix);    },

            replay_drawing:     replay_drawing,
            map_coords:         map_coords,
            draw_layers:        draw_layers,

            stats:              ()              => { 
                let result = {
                    replay_length:      replay.length, 
                    num_layers:         (layer_canvases ? Object.keys(layer_canvases).length : 1)
                };
                return result;
            }
        };
    }

    ///
    /// Creates a decoder that will accept a string of serialized canvas data and
    /// draw it using the provided set of drawing functions
    ///
    function create_decoder(draw) {
        let decoder = (serialized_instructions) => {
            // Position in the instruction set
            let pos             = 0;

            // DataView for decoding floats
            let float_buffer    = new ArrayBuffer(24);
            let float_bytes     = new Uint8Array(float_buffer);
            let float_data      = new DataView(float_buffer);

            ///
            /// Reads a single character from the instructions
            ///
            let read_char = () => {
                let result = null;

                if (pos < serialized_instructions.length) {
                    result = serialized_instructions[pos];
                }
                ++pos;

                return result;
            };

            ///
            /// Returns the value for a particular character fragment
            ///
            let char_code_A = 'A'.charCodeAt(0);
            let char_code_a = 'a'.charCodeAt(0);
            let char_code_0 = '0'.charCodeAt(0);
            let fragment_val = (fragment_char) => {
                let char_code = fragment_char.charCodeAt(0);
                if (fragment_char >= 'A' && fragment_char <= 'Z') {
                    return char_code - char_code_A;
                } else if (fragment_char >= 'a' && fragment_char <= 'z') {
                    return char_code - char_code_a + 26;
                } else if (fragment_char >= '0' && fragment_char <= '9') {
                    return char_code - char_code_0 + 52;
                } else if (fragment_char === '+') {
                    return 62;
                } else if (fragment_char === '/') {
                    return 63;
                } else {
                    return 0;
                }
            };

            ///
            /// Reads a 4-byte word into the buffer at the specified offset
            ///
            let buffer_word = (offset) => {
                // Do nothing if we overrun the end of the buffer
                if (pos + 6 > serialized_instructions.length) {
                    return;
                }
                
                // Read a fragment
                let fragment = serialized_instructions.substring(pos, pos+6);
                pos += 6;

                // Decode it
                let code_point = [ 0,0,0,0,0,0 ];
                for (let p = 0; p<6; ++p) {
                    code_point[p] = fragment_val(fragment[p]);
                }

                float_bytes[offset+3] = (code_point[0])     | ((code_point[1]&0x3)<<6);
                float_bytes[offset+2] = (code_point[1]>>2)  | ((code_point[2]&0xf)<<4);
                float_bytes[offset+1] = (code_point[2]>>4)  | (code_point[3]<<2);
                float_bytes[offset+0] = (code_point[4])     | ((code_point[5]&0x3)<<6);
            };

            ///
            /// Reads a floating point value
            ///
            let read_float = () => {
                buffer_word(0);
                return float_data.getFloat32(0);
            };

            ///
            /// Reads an unsigned int value
            ///
            let read_u32 = () => {
                buffer_word(0);
                return float_data.getUint32(0);
            };

            ///
            /// Reads a u64 stored in 'truncated' format
            ///
            let read_truncated_u64 = () => {
                let result  = 0;
                let shift   = 0;

                for (;;) {
                    let next_val    = fragment_val(read_char());
                    let val_part    = next_val & 0x1f;
                    let more_vals   = next_val & 0x20;

                    result |= val_part << shift;

                    if (more_vals === 0) {
                        break;
                    }

                    shift += 5;
                    if (shift >= 64) { break; }
                }

                return result;
            };

            let read_sprite_id = read_truncated_u64;

            ///
            /// Reads a RGBA colour
            ///
            let read_rgba = () => {
                let color_type = read_char();

                switch (color_type) {
                case 'R':   return [ read_float(), read_float(), read_float(), read_float() ];
                default:    throw 'Unknown color type: \'' + color_type + '\'';
                }
            };

            ///
            /// Decodes a 'new' instruction
            ///
            let decode_new = () => {
                switch (read_char()) {
                case 'p':   draw.new_path();        break;
                case 'A':   draw.clear_canvas();    break;
                case 'l':   draw.layer(read_u32()); break;
                case 'b':   draw.layer_blend(read_u32(), decode_blend_mode()); break;
                case 'C':   draw.clear_layer();     break;
                case 's':   draw.sprite(read_sprite_id()); break;
                }
            };

            ///
            /// Decodes a colour operation
            ///
            let decode_color = () => {
                let color_target    = read_char();
                let color           = read_rgba();

                switch (color_target) {
                case 's':   draw.stroke_color(color[0], color[1], color[2], color[3]);  break;
                case 'f':   draw.fill_color(color[0], color[1], color[2], color[3]);    break;
                default:    throw 'Unknown color target: \'' + color_target + '\'';
                }
            };

            ///
            /// Decodes a line properties command
            ///
            let decode_line = () => {
                switch (read_char()) {
                case 'w':   draw.line_width(read_float());          break;
                case 'p':   draw.line_width_pixels(read_float());   break;
                case 'j':
                    switch (read_char()) {
                    case 'M':   draw.line_join('miter'); break;
                    case 'R':   draw.line_join('round'); break;
                    case 'B':   draw.line_join('bevel'); break;
                    }
                    break;
                case 'c':
                    switch (read_char()) {
                    case 'B':   draw.line_cap('butt');      break;
                    case 'R':   draw.line_cap('round');     break;
                    case 'S':   draw.line_cap('square');    break;
                    }
                    break;
                }
            };

            let decode_blend_mode = () => {
                switch (read_char()) {
                case 'S':
                    switch (read_char()) {
                    case 'V':   draw.blend_mode('source-over'); break;
                    case 'I':   draw.blend_mode('source-in');   break;
                    case 'O':   draw.blend_mode('source-out');  break;
                    case 'A':   draw.blend_mode('source-atop'); break;
                    }
                    break;
                case 'D':
                    switch (read_char()) {
                    case 'V':   draw.blend_mode('destination-over');    break;
                    case 'I':   draw.blend_mode('destination-in');      break;
                    case 'O':   draw.blend_mode('destination-out');     break;
                    case 'A':   draw.blend_mode('destination-atop');    break;
                    }
                    break;
                case 'E':
                    switch (read_char()) {
                    case 'M':   draw.blend_mode('multiply');    break;
                    case 'S':   draw.blend_mode('screen');      break;
                    case 'D':   draw.blend_mode('darken');      break;
                    case 'L':   draw.blend_mode('lighten');     break;
                    }
                    break;
                }
            };

            let decode_transform    = () => {
                switch (read_char()) {
                case 'i':   draw.identity_transform(); break;
                case 'h':   draw.canvas_height(read_float()); break;
                case 'c':   draw.center_region(read_float(), read_float(), read_float(), read_float()); break;
                case 'm':   
                    {
                        let transform = [ 1,0,0, 0,1,0, 0,0,1 ];
                        for (let p=0; p<9; ++p) transform[p] = read_float();
                        draw.multiply_transform(transform);
                    }
                    break;
                }
            };

            let read_matrix = () => {
                let transform = [ 1,0,0, 0,1,0, 0,0,1 ];
                for (let p=0; p<9; ++p) transform[p] = read_float();
                return transform;
            };

            let decode_clip = () => {
                switch (read_char()) {
                case 'c':   draw.clip();                break;
                case 'n':   draw.unclip();              break;
                case 's':   draw.store();               break;
                case 'r':   draw.restore();             break;
                case 'f':   draw.free_stored_buffer();  break;
                }
            };

            let decode_sprite = () => {
                switch (read_char()) {
                case 'C':   draw.clear_sprite();        break;
                case 'T':   decode_sprite_transform();  break;
                case 'D':   draw.draw_sprite(read_sprite_id()); break;
                }
            };

            let decode_sprite_transform = () => {
                switch (read_char()) {
                case 'i':   draw.sprite_transform_identity();   break;
                case 't':   draw.sprite_transform_translate(read_float(), read_float());    break;
                case 's':   draw.sprite_transform_scale(read_float(), read_float());        break;
                case 'r':   draw.sprite_transform_rotate(read_float());                     break;
                case 'T':   draw.sprite_transform_matrix(read_matrix());                    break;
                }
            };
            
            let decode_dash         = () => { throw 'Not implemented'; };
            
            for(;;) {
                let instruction = read_char();

                if (instruction === null) break;

                switch (instruction) {
                case ' ':
                case '\n':
                    break;
                
                case 'N':   decode_new();                               break;
                case 'm':   draw.move_to(read_float(), read_float());   break;
                case 'l':   draw.line_to(read_float(), read_float());   break;
                case 'c':   draw.bezier_curve(read_float(), read_float(), read_float(), read_float(), read_float(), read_float()); break;
                case '.':   draw.close_path();                          break;
                case 'F':   draw.fill();                                break;
                case 'S':   draw.stroke();                              break;
                case 'L':   decode_line();                              break;
                case 'D':   decode_dash();                              break;
                case 'C':   decode_color();                             break;
                case 'M':   decode_blend_mode();                        break;
                case 'T':   decode_transform();                         break;
                case 'Z':   decode_clip();                              break;
                case 'P':   draw.push_state();                          break;
                case 'p':   draw.pop_state();                           break;
                case 's':   decode_sprite();                            break;

                default:    throw 'Unknown instruction \'' + instruction + '\' at ' + pos;
                }
            }

            draw.draw_layers();
        };

        return decoder;
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
        // The canvas map will need to be updated before we can look up canvases by name
        mark_canvases_outdated();

        let draw = canvas.flo_draw;

        ///
        /// Returns true if this canvas is active
        ///
        let is_active = () => {
            return canvas.parentNode !== null;
        };

        ///
        /// Updates the content size of the canvas
        ///
        let resize_canvas = () => {
            // Resize if the canvas's size has changed
            var ratio           = 1; //window.devicePixelRatio || 1; -- TODO, 4k canvases cause the pointer events to lag on Chrome...
            let target_width    = canvas.clientWidth * ratio;
            let target_height   = canvas.clientHeight * ratio;

            if (canvas.width !== target_width || canvas.height !== target_height) {
                // Actually resize the canvas
                canvas.width    = canvas.clientWidth * ratio;
                canvas.height   = canvas.clientHeight * ratio;

                // Redraw the canvas contents at the new size
                draw.replay_drawing();
                draw.draw_layers();
            }
        };

        // Add this canvas to the list of active canvases
        active_canvases.push({
            canvas_element: canvas,
            is_active:      is_active,
            resize_canvas:  resize_canvas,
            draw:           canvas.flo_draw,
            flo_name:       canvas.flo_name,
            flo_controller: canvas.flo_controller,
            decoder:        canvas.flo_canvas_decoder
        });

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
    /// Returns the canvas with the specified path
    ///
    function get_canvas(controller_path, canvas_name) {
        // Need the canvas map to be up to date
        if (canvas_map_outdated) {
            update_canvas_map();
        }

        // Return the canvas at this address
        let address = controller_path + '/' + canvas_name;
        return canvas_map[address];
    }

    ///
    /// Attempts to ressurect a canvas from the boneyard (saves reloading a canvas that's only mostly dead)
    ///
    function get_zombie_canvas(controller_path, canvas_name) {
        for (let index = 0; index < boneyard.length; ++index) {
            // Find the ex-canvas
            let dead_canvas = boneyard[index];
            if (!dead_canvas) {
                continue;
            }

            // Resurrect if it's a Norwegian Blue
            if (dead_canvas.flo_controller === controller_path && dead_canvas.flo_name === canvas_name) {
                // Lurch down to the village
                boneyard[index] = null;
                return dead_canvas;
            }
        }

        return null;
    }

    ///
    /// Updates the canvas with the specified path using an encoded update
    ///
    function update_canvas(controller_path, canvas_name, encoded_update) {
        // Fetch the canvas with this name
        let canvas = get_canvas(controller_path, canvas_name);

        if (!canvas) {
            // Error if the canvas doesn't exist
            console.error('Canvas ' + controller_path + '/' + canvas_name + ' could not be found during update');
        } else {
            // Send the update to the canvas decoder
            try {
                canvas.decoder(encoded_update);
            } catch (e) {
                console.error('Could not decode ', encoded_update);
                throw e;
            }
        }
    }

    ///
    /// Uses a zombie canvas (canvas element which no longer in the DOM) to create a new canvas for an element
    ///
    function resurrect_canvas(element, zombie) {
        let parent = element;

        // Read the canvas attributes
        let flo_controller  = element.getAttribute('flo-controller');
        let flo_name        = element.getAttribute('flo-name');

        // Get the original canvas element
        let canvas = zombie.canvas_element;

        // Add to the DOM
        parent.appendChild(canvas);

        // Set up the element
        element.flo_canvas_decoder  = canvas.flo_canvas_decoder;
        element.flo_draw            = canvas.flo_draw;
        element.flo_map_coords      = canvas.flo_map_coords;
        element.flo_controller      = flo_controller;
        element.flo_name            = flo_name;

        // Restart monitoring events for this canvas
        monitor_canvas_events(canvas);

        return {
            canvas: canvas,
            draw:   canvas.flo_draw
        };
    }

    ///
    /// Creates a canvas for an element
    ///
    function create_canvas(element) {
        let parent = element;

        // Read the canvas attributes
        let flo_controller  = element.getAttribute('flo-controller');
        let flo_name        = element.getAttribute('flo-name');

        // Create a new canvas element
        let canvas = document.createElement('canvas');

        // Add it to the DOM
        parent.appendChild(canvas);

        // Set up the element
        let draw                    = create_drawing_functions(canvas);
        let decoder                 = create_decoder(draw);
        element.flo_canvas_decoder  = decoder;
        element.flo_draw            = draw;
        element.flo_map_coords      = draw.map_coords;
        element.flo_controller      = flo_controller;
        element.flo_name            = flo_name;
        canvas.flo_draw             = draw;
        canvas.flo_canvas_decoder   = decoder;
        canvas.flo_name             = flo_name;
        canvas.flo_controller       = flo_controller;
        canvas.flo_map_coords       = draw.map_coords;

        apply_canvas_style(canvas);
        monitor_canvas_events(canvas);
        
        // Return the properties to attach to the parent element
        return {
            canvas: canvas,
            draw:   canvas.flo_draw
        };
    }

    // The final flo_canvas object
    return {
        start:                      start,
        stop:                       stop,
        resize_canvases:            resize_active_canvases,
        update_canvas:              update_canvas,
        remove_inactive_canvases:   remove_inactive_canvases,
        update_canvas_map:          update_canvas_map
    };
})();
