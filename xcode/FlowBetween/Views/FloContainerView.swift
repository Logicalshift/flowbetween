//
//  FloContainer.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 06/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

///
/// Represents the visibility of the scrollbars
///
enum ScrollBarVisibility {
    case Never;
    case Always;
    case OnlyIfNeeded;
}

///
/// Protocol implemented by views that can act as container views
///
protocol FloContainerView {
    /// Adds a subview to this container view
    func addContainerSubview(_ subview: NSView);
    
    /// Sets the layer displayed for the canvas
    func setCanvasLayer(_ layer: CALayer);
    
    /// The size of the layout area for this view
    var layoutSize : NSSize { get };

    /// Returns this view as an NSView
    var asView : NSView { get };
    
    /// Event handler: user clicked in the view
    var onClick: (() -> Bool)? { get set };
    
    /// Event handler: user scrolled/resized so that a particular region is visible
    var onScroll: ((NSRect) -> ())? { get set };
    
    /// Events handlers when a particular device is used for painting
    var onPaint: [FloPaintDevice: (FloPaintStage, AppPainting) -> ()] { get set };

    /// The affine transform for the canvas layer
    var canvasAffineTransform: CGAffineTransform? { get set }

    /// Event handler: user performed layout on this view
    var performLayout: ((NSSize) -> ())? { get set };
    
    /// Event handler: The bounds of the container have changed
    var boundsChanged: ((ContainerBounds) -> ())? { get set };
    
    /// The minimum size of the scroll area for this view
    var scrollMinimumSize: (Float64, Float64) { get set }
    
    /// The visibility of the horizontal and vertical scroll bars
    var scrollBarVisibility: (ScrollBarVisibility, ScrollBarVisibility) { get set }
    
    /// Triggers the click event for this view
    func triggerClick();
    
    /// Triggers the bounds changed event for this view
    func triggerBoundsChanged();
    
    /// Sets the text label for this view
    func setTextLabel(label: String);
    
    /// Sets the font size for this view
    func setFontSize(points: Float64);
    
    /// Sets the font weight for this view
    func setFontWeight(weight: Float64);
    
    /// Sets the text alignment for this view
    func setTextAlignment(alignment: NSTextAlignment);
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
