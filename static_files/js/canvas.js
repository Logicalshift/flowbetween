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

    // Maps controller_path + canvas_name to the list of active canvases
    let canvas_map      = {};

    // True if the canvas map is outdated
    let canvas_map_outdated = true;

    ///
    /// Remove any canvas from the list of active canvases that no longer have a parent
    ///
    function remove_inactive_canvases() {
        // Remove any canvas element that has a null parent
        for (let index=0; index<active_canvases.length; ++index) {
            if (!active_canvases[index].is_active()) {
                active_canvases.splice(index, 1);
                --index;
                mark_canvases_outdated();
            }
        }
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
                }
            });
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
        // The replay log will replay the actions that draw this canvas (for example when resizing)
        let replay  = [ [ clear_canvas, [] ] ];

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

        function new_path()                             { context.beginPath(); current_path = []; }
        function move_to(x,y)                           { context.moveTo(x, y); current_path.push(() => context.moveTo(x, y) ); }
        function line_to(x,y)                           { context.lineTo(x, y); current_path.push(() => context.lineTo(x, y) ); }
        function bezier_curve(x1, y1, x2, y2, x3, y3)   { context.bezierCurveTo(x2, y2, x3, y3, x1, y1); current_path.push(() => context.bezierCurveTo(x2, y2, x3, y3, x1, y1) ); }
        function close_path()                           { context.closePath(); }
        function fill()                                 { context.fill(); }

        function stroke() {
            if (set_dash_pattern) {
                set_dash_pattern = false;
                context.setLineDash(dash_pattern);
            }

            context.stroke(); 
        }

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
            context.lineWidth = width;
        }

        function line_width_pixels(width) {
            // Length of the first column of the transformation matrix is the scale factor (for the width)
            let scale = Math.sqrt(transform[0]*transform[0] + transform[3]*transform[3]);
            if (scale === 0) scale = 1;
            scale /= window.devicePixelRatio || 1;

            // Scale the width down according to this factor (we'll always use the horizontal scale factor)
            context.lineWidth = width / scale;
        }

        function line_join(join) {
            context.lineJoin = join;
        }

        function line_cap(cap) {
            context.lineCap = cap;
        }

        function new_dash_pattern() {
            dash_pattern        = [];
            set_dash_pattern    = true;
        }

        function dash_length(length) {
            context.lineDashOffset = length;
        }

        function dash_offset(offset) {
            dash_pattern.push(offset);
            set_dash_pattern = true;
        }

        function blend_mode(blend_mode) {
            context.globalCompositeOperation = blend_mode;
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

            transform_set([ 
                ratio,  0,      pixel_width/2.0,
                0,      -ratio, pixel_height/2.0, 
                0,      0,      1
            ]);
        }

        function center_region(minx, miny, maxx, maxy) {
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

            multiply_transform([
                1, 0, x_offset,
                0, 1, y_offset,
                0, 0, 1
            ]);
        }

        function multiply_transform(transform) {
            // Rotated transformation matrix
            context.transform(transform[0], transform[3], transform[1], transform[4], transform[2], transform[5]);
            transform_multiply(transform);
        }

        ///
        /// Removes the clipping path if one is applied
        ///
        function remove_clip() {
            if (clipped) {
                clipped = false;
                context.restore();
            }
        }

        ///
        /// Restores the clipping path if it's missing
        ///
        function restore_clip() {
            if (!clipped) {
                clipped = true;
                context.save();
                clip_stack.forEach(fn => fn());
            }
        }

        function unclip() {
            // Stop clipping and clear the stack
            remove_clip();
            clip_stack = [];
        }

        function clip() {
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
        }

        function store() {
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
        }

        function restore() {
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

                last_store_pos      = null;
                have_stored_image   = false;
            }
        }

        function push_state() {
            // Push the current clipping path and dash pattern
            let restore_clip_stack      = clip_stack.slice();
            let restore_dash_pattern    = dash_pattern.slice();
            let restore_stored_pixels   = stored_pixels;
            let restore_gen_buffer      = generate_buffer_on_store;
            let restore_have_image      = have_stored_image;
            let restore_layer_id        = current_layer_id;
            let restore_last_store_pos  = last_store_pos;
            context_stack.push(() => {
                clip_stack                  = restore_clip_stack;
                dash_pattern                = restore_dash_pattern;
                stored_pixels               = restore_stored_pixels;
                generate_buffer_on_store    = restore_gen_buffer;
                have_stored_image           = restore_have_image;
                current_layer_id            = restore_layer_id;
                last_store_pos              = restore_last_store_pos;
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
        }

        function pop_state() {
            // Remove any clipping we have
            remove_clip();

            // Restore state not saved in context
            context_stack.pop()();

            // Restore context state
            context.restore();

            // Reinstate the clipping
            restore_clip();
        }

        function create_layer() {
            let new_layer       = document.createElement('canvas');
            new_layer.width     = canvas.width;
            new_layer.height    = canvas.height;

            return new_layer;
        }

        function copy_canvas(src_canvas, target_canvas) {
            let target_context = target_canvas.getContext('2d');

            target_context.save();
            target_context.setTransform(1,0, 0,1, 0,0);
            target_context.globalCompositeOperation = 'copy';

            target_context.drawImage(src_canvas, 0,0, src_canvas.width, src_canvas.height);

            target_context.restore();
        }

        function layer(layer_id) {
            // Clear any existing clipping rect
            unclip();

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
        }

        function layer_blend(layer_id, blend_mode) {
            blend_for_layer[layer_id] = blend_mode;
        }

        function clear_canvas() {
            // Clear layers
            layer_canvases      = null;
            context             = canvas.getContext('2d');
            blend_for_layer     = {};
            current_layer_id    = 0;

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
        }

        function rewind_to_last_store() {
            if (last_store_pos !== null) {
                while (replay.length >= last_store_pos) {
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

                // Draw each of the layers
                Object.keys(layer_canvases).forEach(layer_id => {
                    layer_context.globalCompositeOperation = blend_for_layer[layer_id] || 'source-over';
                    layer_context.drawImage(layer_canvases[layer_id], 0,0, width,height);
                });
            }
        }

        return {
            new_path:           ()              => { replay.push([new_path, []]);                       new_path();                     },
            move_to:            (x, y)          => { replay.push([move_to, [x, y]]);                    move_to(x, y);                  },
            line_to:            (x, y)          => { replay.push([line_to, [x, y]]);                    line_to(x, y);                  },
            bezier_curve:       (x1, y1, x2, y2, x3, y3) => { replay.push([bezier_curve, [x1, y1, x2, y2, x3, y3]]); bezier_curve(x1, y1, x2, y2, x3, y3); },
            close_path:         ()              => { replay.push([close_path, []]);                     close_path();                   },
            fill:               ()              => { replay.push([fill, []]);                           fill();                         },
            stroke:             ()              => { replay.push([stroke, []]);                         stroke();                       },
            line_width:         (width)         => { replay.push([line_width, [width]]);                line_width(width);              },
            line_width_pixels:  (width)         => { replay.push([line_width_pixels, [width]]);         line_width_pixels(width);       },
            line_join:          (join)          => { replay.push([line_join, [join]]);                  line_join(join);                },
            line_cap:           (cap)           => { replay.push([line_cap, [cap]]);                    line_cap(cap);                  },
            new_dash_pattern:   ()              => { replay.push([new_dash_pattern, []]);               new_dash_pattern();             },
            dash_length:        (length)        => { replay.push([dash_length, [length]]);              dash_length(length);            },
            dash_offset:        (offset)        => { replay.push([dash_offset, [offset]]);              dash_length(offset);            },
            fill_color:         (r, g, b, a)    => { replay.push([fill_color, [r, g, b, a]]);           fill_color(r, g, b, a);         },
            stroke_color:       (r, g, b, a)    => { replay.push([stroke_color, [r, g, b, a]]);         stroke_color(r, g, b, a);       },
            blend_mode:         (mode)          => { replay.push([blend_mode, [mode]]);                 blend_mode(mode);              },
            identity_transform: ()              => { replay.push([identity_transform, []]);             identity_transform();           },
            canvas_height:      (height)        => { replay.push([canvas_height, [height]]);            canvas_height(height);          },
            center_region:      (x1, y1, x2, y2) => { replay.push([center_region, [x1, y1, x2, y2]]);   center_region(x1, y1, x2, y2);  },
            multiply_transform: (transform)     => { replay.push([multiply_transform, [transform]]);    multiply_transform(transform);  },
            unclip:             ()              => { replay.push([unclip, []]);                         unclip();                       },
            clip:               ()              => { replay.push([clip, []]);                           clip();                         },
            store:              ()              => { replay.push([store, []]);                          store();                        },
            restore:            ()              => { replay.push([restore, []]); rewind_to_last_store(); restore();                     },
            push_state:         ()              => { replay.push([push_state, []]);                     push_state();                   },
            pop_state:          ()              => { replay.push([pop_state, []]);                      pop_state();                    },
            layer:              (layer_id)      => { replay.push([layer, [layer_id]]);                  layer(layer_id);                },
            layer_blend:        (layer_id, blend_mode) => { replay.push([layer_blend, [layer_id, blend_mode]]); layer_blend(layer_id, blend_mode); },
            clear_canvas:       ()              => { replay = [ [clear_canvas, []] ];                   clear_canvas();                 },

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

            let decode_clip = () => {
                switch (read_char()) {
                case 'c':   draw.clip();    break;
                case 'n':   draw.unclip();  break;
                case 's':   draw.store();   break;
                case 'r':   draw.restore(); break;
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

        // Add this canvas to the list of active canvases
        active_canvases.push({
            is_active:      is_active,
            resize_canvas:  resize_canvas,
            draw:           canvas.flo_draw,
            flo_name:       canvas.flo_name,
            flo_controller: canvas.flo_controller,
            decoder:        canvas.flo_canvas_decoder
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
                draw.draw_layers();
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
            shadow: shadow,
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
