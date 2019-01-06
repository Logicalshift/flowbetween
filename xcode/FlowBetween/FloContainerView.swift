//
//  FloContainer.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 06/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

///
/// Protocol implemented by views that can act as container views
///
protocol FloContainerView {
    /// Returns this view as an NSView
    var asView : NSView { get };
    
    /// Event handler: user clicked in the view
    var onClick: (() -> Bool)? { get set };
    
    /// Event handler: user performed layout on this view
    var performLayout: (() -> ())? { get set };
    
    /// Triggers the click event for this view
    func triggerClick();
}

///
/// Bubbles an event up from a particular view
///
func bubble_up_event(source: NSView, event_handler: (FloContainerView) -> Bool) {
    // Bubble up to the superview
    var bubble_to: NSView? = source;
    
    while true {
        if let bubble_to_view = bubble_to {
            // Try this view
            if let bubble_to = bubble_to_view as? FloContainerView {
                if event_handler(bubble_to) {
                    // Event was handled
                    return;
                }
            }
            
            // Try the superview
            bubble_to = bubble_to_view.superview;
        } else {
            // Did not find a target
            return;
        }
    }
}
