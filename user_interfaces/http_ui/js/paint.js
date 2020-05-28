'use strict';

//   __ _                    _     _
//  / _| |___ ___ _ __  __ _(_)_ _| |_
// |  _| / _ \___| '_ \/ _` | | ' \  _|
// |_| |_\___/   | .__/\__,_|_|_||_\__|
//               |_|

//
// Interpreting pointer/mouse/touch events is hard enough we need a whole file to
// do it in.
//
// We prefer pointer events, but will fall back to touch events and the mozPressure
// extension depending on what's available. Browser support for pressure on different
// platforms is highly variable.
//

/* exported flo_paint */

let flo_paint = (function() {
    // Test for browser functionality
    let supports_pointer_events = 'onpointerdown' in window;
    let supports_touch_events   = 'ontouchstart' in window;

    ///
    /// Converts a MouseEvent to a Paint object.
    ///
    /// Action should be 'Start', 'Continue' or 'Finish'
    ///
    let mouse_event_to_paint_event = (mouse_event, action, target_element) => {
        // Get the coordinates of this event
        let x = mouse_event.clientX;
        let y = mouse_event.clientY;

        // Get the target element
        let client_rect = target_element.getBoundingClientRect();

        x -= client_rect.left;
        y -= client_rect.top;

        // Re-map the coordinates if the target element has any to map
        if (target_element.flo_map_coords) {
            let coords = target_element.flo_map_coords(x, y);
            x = coords[0];
            y = coords[1];
        }

        // Generate the final event
        // TODO: can get tilt_x and tilt_y from azimuthAngle and altitudeAngle (but they aren't a direct mapping)
        return {
            action:     action,
            pointer_id: 0,
            location:   [x, y],
            pressure:   mouse_event.mozPressure || 0.5,
            tilt_x:     0,
            tilt_y:     0
        };
    };

    ///
    /// Converts a TouchEvent to a Paint object.
    ///
    /// Action should be 'Start', 'Continue' or 'Finish'
    ///
    let touch_event_to_paint_event = (touch_event, action, target_element) => {
        // We always just track the first touch here
        let touch = touch_event.touches[0];

        // Get the coordinates of this event
        let x = touch.clientX;
        let y = touch.clientY;

        // Get the target element
        let client_rect = target_element.getBoundingClientRect();
        
        x -= client_rect.left;
        y -= client_rect.top;
        
        // Re-map the coordinates if the target element has any to map
        if (target_element.flo_map_coords) {
            let coords = target_element.flo_map_coords(x, y);
            x = coords[0];
            y = coords[1];
        }

        // Generate the final event
        // TODO: can get tilt_x and tilt_y from azimuthAngle and altitudeAngle (but they aren't a direct mapping)
        return {
            action:     action,
            pointer_id: 0,
            location:   [x, y],
            pressure:   touch.force || 0.5,
            tilt_x:     0,
            tilt_y:     0
        };
    };

    ///
    /// Converts a PointerEvent to a Paint object.
    ///
    /// Action should be 'Start', 'Continue' or 'Finish'
    ///
    let pointer_event_to_paint_event = (pointer_event, action, target_element) => {
        // Get the coordinates of this event
        let x = pointer_event.clientX;
        let y = pointer_event.clientY;

        // Get the target element
        let client_rect = target_element.getBoundingClientRect();
        
        x -= client_rect.left;
        y -= client_rect.top;
        
        // Re-map the coordinates if the target element has any to map
        if (target_element.flo_map_coords) {
            let coords = target_element.flo_map_coords(x, y);
            x = coords[0];
            y = coords[1];
        }

        // Generate the final event
        return {
            action:     action,
            pointer_id: pointer_event.pointerId,
            location:   [x, y],
            pressure:   pointer_event.pressure,
            tilt_x:     pointer_event.tiltX,
            tilt_y:     pointer_event.tiltY
        };
    };


    ///
    /// Wires up a paint action to a node using the mouse events API
    ///
    let wire_paint_mouse_events = (target_device, action_name, node, controller_path) => {
        if (!target_device['Mouse']) {
            return;
        }

        let check_device = () => true;
        switch (target_device['Mouse']) {
        case 'Left':    check_device = mouse_event => mouse_event.button === 0; break;
        case 'Middle':  check_device = mouse_event => mouse_event.button === 1; break;
        case 'Right':   check_device = mouse_event => mouse_event.button === 2; break;
        case 'Other':   check_device = mouse_event => mouse_event.button > 2; break;
        }
        
        // The in-flight event is used to queue events while we wait for FlowBetween to process existing events
        let in_flight_event = new Promise((resolve) => resolve());

        // The waiting events are move events that have arrived before the in-flight event finished
        let waiting_events  = [];

        // Declare our event handlers
        let mouse_down = mouse_event => {
            // Must be the right mouse button
            if (!check_device(mouse_event)) {
                return;
            }

            // Start tracking this touch event
            document.addEventListener('mousemove', mouse_move, true);
            document.addEventListener('mouseup', mouse_up, true);
            
            mouse_event.preventDefault();

            // Create the 'start' event
            let start_parameter = {
                Paint: [
                    target_device,
                    [ mouse_event_to_paint_event(mouse_event, 'Start', node) ]
                ]
            };

            in_flight_event = in_flight_event.then(() => perform_action(controller_path, action_name, start_parameter));
        };

        let mouse_move = mouse_event => {
            mouse_event.preventDefault();

            waiting_events.push(mouse_event_to_paint_event(mouse_event, 'Continue', node));

            // Send the move event as soon as the in-flight events have finished processing
            in_flight_event = in_flight_event.then(() => {
                if (waiting_events.length > 0) {
                    let move_parameter = {
                        Paint: [
                            target_device,
                            waiting_events
                        ]
                    };
                    waiting_events = [];
                    return perform_action(controller_path, action_name, move_parameter);
                }
            });
        };

        let mouse_up = mouse_event => {
            mouse_event.preventDefault();

            // We finish using the last event as the 'end' event will have 0 touches
            let finish_parameter = {
                Paint: [
                    target_device,
                    [ mouse_event_to_paint_event(mouse_event, 'Finish', node) ]
                ]
            };
            in_flight_event = in_flight_event.then(() => perform_action(controller_path, action_name, finish_parameter));

            // Release the device
            document.removeEventListener('mousemove', mouse_move, true);
            document.removeEventListener('mouseup', mouse_up, true);
        };

        // Register for the pointer down event
        add_action_event(node, 'mousedown', mouse_down, false);
    };

    ///
    /// Wires up a paint action to a node using the touch events API
    ///
    let wire_paint_touch_events = (target_device, action_name, node, controller_path) => {
        // We only wire for 'touch' events when using the touch API
        if (target_device !== 'Touch' && target_device !== 'Pen') {
            return;
        }
        let needs_stylus = target_device === 'Pen';

        // The in-flight event is used to queue events while we wait for FlowBetween to process existing events
        let in_flight_event = new Promise((resolve) => resolve());

        // The waiting events are move events that have arrived before the in-flight event finished
        let waiting_events  = [];

        // The last event is used to finish a touch
        let last_event      = null;

        // Declare our event handlers
        let touch_start = touch_event => {
            let touch = touch_event.touches[0];

            if (touch.touchType === 'stylus') {
                if (!needs_stylus) {
                    return;
                }
            } else {
                if (needs_stylus) {
                    return;
                }
            }

            // Start tracking this touch event
            document.addEventListener('touchmove', touch_move, true);
            document.addEventListener('touchend', touch_end, true);
            document.addEventListener('touchcancel', touch_cancel, true);
            
            touch_event.preventDefault();

            // Create the 'start' event
            last_event = touch_event;
            let start_parameter = {
                Paint: [
                    target_device,
                    [ touch_event_to_paint_event(touch_event, 'Start', node) ]
                ]
            };

            in_flight_event = in_flight_event.then(() => perform_action(controller_path, action_name, start_parameter));
        };

        let touch_move = touch_event => {
            touch_event.preventDefault();

            last_event = touch_event;
            waiting_events.push(touch_event_to_paint_event(touch_event, 'Continue', node));

            // Send the move event as soon as the in-flight events have finished processing
            in_flight_event = in_flight_event.then(() => {
                if (waiting_events.length > 0) {
                    let move_parameter = {
                        Paint: [
                            target_device,
                            waiting_events
                        ]
                    };
                    waiting_events = [];
                    return perform_action(controller_path, action_name, move_parameter);
                }
            });
        };

        let touch_end = touch_event => {
            touch_event.preventDefault();

            // We finish using the last event as the 'end' event will have 0 touches
            let finish_parameter = {
                Paint: [
                    target_device,
                    [ touch_event_to_paint_event(last_event, 'Finish', node) ]
                ]
            };
            in_flight_event = in_flight_event.then(() => perform_action(controller_path, action_name, finish_parameter));

            // Release the device
            document.removeEventListener('touchmove', touch_move, true);
            document.removeEventListener('touchend', touch_end, true);
            document.removeEventListener('touchcancel', touch_end, true);
        };

        let touch_cancel = touch_event => {
            touch_event.preventDefault();

            // We finish using the last event as the 'end' event will have 0 touches
            let finish_parameter = {
                Paint: [
                    target_device,
                    [ touch_event_to_paint_event(last_event, 'Cancel', node) ]
                ]
            };
            in_flight_event = in_flight_event.then(() => perform_action(controller_path, action_name, finish_parameter));

            // Release the device
            document.removeEventListener('touchmove', touch_move, true);
            document.removeEventListener('touchend', touch_end, true);
            document.removeEventListener('touchcancel', touch_end, true);
        };

        // Register for the pointer down event
        add_action_event(node, 'touchstart', touch_start, true);
    };

    ///
    /// Wires up a paint action to a node using the pointerevents API
    ///
    let wire_paint_pointer_events = (target_device, action_name, node, controller_path) => {
        // Function to check if a pointer event is for the right device
        let check_device = () => true;
        if (target_device === 'Pen')            { check_device = pointer_event => pointer_event.pointerType === 'pen' && pointer_event.button !== 5; }
        else if (target_device === 'Eraser')    { check_device = pointer_event => pointer_event.pointerType === 'pen' && pointer_event.button === 5; }
        else if (target_device === 'Touch')     { check_device = pointer_event => pointer_event.pointerType === 'touch'; }
        else if (target_device['Mouse']) {
            switch (target_device['Mouse']) {
            case 'Left':    check_device = pointer_event => pointer_event.pointerType === 'mouse' && pointer_event.button === 0; break;
            case 'Middle':  check_device = pointer_event => pointer_event.pointerType === 'mouse' && pointer_event.button === 1; break;
            case 'Right':   check_device = pointer_event => pointer_event.pointerType === 'mouse' && pointer_event.button === 2; break;
            case 'Other':   check_device = pointer_event => pointer_event.pointerType === 'mouse' && pointer_event.button > 2; break;
            }
        } else if (target_device === 'Other') {
            check_device = pointer_event => {
                switch (pointer_event.pointerType) {
                case 'pen':
                case 'touch':
                case 'mouse':
                    return false;
                
                default:
                    return true;
                }
            };
        }

        // The device that we're currently tracking
        let pointer_device  = '';

        // The in-flight event is used to queue events while we wait for FlowBetween to process existing events
        let in_flight_event = new Promise((resolve) => resolve());

        // The waiting events are move events that have arrived before the in-flight event finished
        let waiting_events  = [];

        // Declare our event handlers
        let pointer_down = pointer_event => {
            if (check_device(pointer_event)) {
                if (pointer_device !== '') {
                    // Already painting
                    pointer_event.preventDefault();
                } else {
                    // Start tracking this pointer event
                    pointer_device = pointer_event.pointerType;

                    document.addEventListener('pointermove', pointer_move, true);
                    document.addEventListener('pointerup', pointer_up, true);
                    document.addEventListener('pointercancel', pointer_cancel, true);
                    
                    // Pointer down on the right device
                    pointer_event.preventDefault();
                    node.setPointerCapture(pointer_event.pointerId);

                    // Create the 'start' event
                    let start_parameter = {
                        Paint: [
                            target_device,
                            [ pointer_event_to_paint_event(pointer_event, 'Start', node) ]
                        ]
                    };

                    in_flight_event = in_flight_event.then(() => perform_action(controller_path, action_name, start_parameter));
                }
            }
        };

        let pointer_move = pointer_event => {
            // Prevent the pointer event from firing
            pointer_event.preventDefault();

            if (pointer_device === pointer_event.pointerType) {
                // This move event is directed to this item
                if (pointer_event.getCoalescedEvents) {
                    pointer_event.getCoalescedEvents().forEach(pointer_event => waiting_events.push(pointer_event_to_paint_event(pointer_event, 'Continue', node)));
                } else {
                    waiting_events.push(pointer_event_to_paint_event(pointer_event, 'Continue', node));
                }

                if (pointer_event.getPredictedEvents) {
                    pointer_event.getCoalescedEvents().forEach(pointer_event => waiting_events.push(pointer_event_to_paint_event(pointer_event, 'Prediction', node)));
                }

                // Send the move event as soon as the in-flight events have finished processing
                in_flight_event = in_flight_event.then(() => {
                    if (waiting_events.length > 0) {
                        let move_parameter = {
                            Paint: [
                                target_device,
                                waiting_events
                            ]
                        };
                        waiting_events = [];
                        return perform_action(controller_path, action_name, move_parameter);
                    }
                });
            }
        };

        let pointer_up = pointer_event => {
            // Prevent the pointer event from firing
            pointer_event.preventDefault();

            let finish_parameter = {
                Paint: [
                    target_device,
                    [ pointer_event_to_paint_event(pointer_event, 'Finish', node) ]
                ]
            };
            in_flight_event = in_flight_event.then(() => perform_action(controller_path, action_name, finish_parameter));

            // Release the device
            pointer_device = '';
            node.releasePointerCapture(pointer_event.pointerId);

            document.removeEventListener('pointermove', pointer_move, true);
            document.removeEventListener('pointerup', pointer_up, true);
            document.removeEventListener('pointercancel', pointer_cancel, true);
        };

        let pointer_cancel = pointer_event => {
            // Prevent the pointer event from firing
            pointer_event.preventDefault();

            // This up event is directed to this item
            let finish_parameter = {
                Paint: [
                    target_device,
                    [ pointer_event_to_paint_event(pointer_event, 'Cancel', node) ]
                ]
            };
            in_flight_event = in_flight_event.then(() => perform_action(controller_path, action_name, finish_parameter));

            // Release the device
            pointer_device = '';
            node.releasePointerCapture(pointer_event.pointerId);

            document.removeEventListener('pointermove', pointer_move, true);
            document.removeEventListener('pointerup', pointer_up, true);
            document.removeEventListener('pointercancel', pointer_cancel, true);
        };

        // Register for the pointer down event
        add_action_event(node, 'pointerdown', pointer_down, false);

        // If touch events are also supported, disable them for this control so gestures are disabled

        // Touch & pointer events fight each other :-(
        // Chrome does not have 'ontouchstart', but secretly supports touch events anyway, so we try to register this regardless
        add_action_event(node, 'touchstart', ev => {
            if (pointer_device !== '') {
                // Pointer events that we're tracking should take precedence over touch events
                ev.preventDefault();
            }
        }, false);
    };

    ///
    /// Wires up the 'paint' events to a node
    ///
    let wire_paint = (target_device, action_name, node, controller_path) => {
        if (supports_pointer_events) {
            // Pointer events are the most general way of tracking what's going on with an event
            wire_paint_pointer_events(target_device, action_name, node, controller_path);
        } else if (supports_touch_events) {
            // Touch events are supported in things like Safari on iOS so we need to support those too
            // They lack the multi-device support of pointer events and some of the extra parameters
            wire_paint_touch_events(target_device, action_name, node, controller_path);
            wire_paint_mouse_events(target_device, action_name, node, controller_path);
        } else {
            // Mouse events are supported everywhere.
            // Firefox supports pressure sensitivity via a browser-specific field.
            // Desktop Safari cannot support pressure-sensitivity.
            wire_paint_mouse_events(target_device, action_name, node, controller_path);
        }
    };

    ///
    /// Registers an event to a particular node
    ///
    /// Needs to be initialised (from flowbetween.js)
    ///
    let add_action_event = () => console.warn('add_action_event called before initialise');

    ///
    /// Sends an action as part of the current session
    ///
    /// Needs to be initialised (from flowbetween.js)
    ///
    let perform_action = () => console.warn('perform_action called before initialise');

    let initialise = (new_add_action_event, new_perform_action) => {
        add_action_event    = new_add_action_event;
        perform_action      = new_perform_action;
    };

    return {
        initialise:                     initialise,

        mouse_event_to_paint_event:     mouse_event_to_paint_event,
        touch_event_to_paint_event:     touch_event_to_paint_event,
        pointer_event_to_paint_event:   pointer_event_to_paint_event,
        wire_paint:                     wire_paint,
        supports_pointer_events:        supports_pointer_events,
        supports_touch_events:          supports_touch_events
    };
})();
