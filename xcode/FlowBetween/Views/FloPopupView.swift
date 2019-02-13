//
//  FloPopupView.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 13/02/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

///
/// Flo view that tracks a popup window
///
class FloPopupView : NSView, FloContainerView {
    override var isOpaque: Bool {
        get {
            return false;
        }
    }
    
    /// Adds a subview to this container view
    func addContainerSubview(_ subview: NSView) {
        // TODO!
    }
    
    /// Sets the layer displayed for the canvas
    func setCanvasLayer(_ layer: CALayer) {
        
    }
    
    /// Stores the general state of this view
    var viewState : ViewState = ViewState();
    
    /// The size of the layout area for this view
    var layoutSize : NSSize {
        get {
            // TODO: size of the view in the popup window
            return NSSize(width: 0.0, height: 0.0);
        }
    }
    
    /// Returns this view as an NSView
    var asView : NSView {
        get {
            return self;
        }
    }
    
    /// Event handler: user clicked in the view
    var onClick: (() -> Bool)?;
    
    /// Event handler: user scrolled/resized so that a particular region is visible
    var onScroll: ((NSRect) -> ())?;
    
    /// Event handler: value has changed
    var onEditValue: ((PropertyValue) -> ())?;
    
    /// Event handler: value has been set
    var onSetValue: ((PropertyValue) -> ())?;
    
    /// Event handler: control has obtained keyboard focus
    var onFocused: (() -> ())?;
    
    /// Event handler: user has dragged this control
    var onDrag: ((DragAction, CGPoint, CGPoint) -> ())?;
    
    /// Events handlers when a particular device is used for painting
    var onPaint: [FloPaintDevice: (FloPaintStage, AppPainting) -> ()] = [FloPaintDevice: (FloPaintStage, AppPainting) -> ()]();
    
    /// The affine transform for the canvas layer
    var canvasAffineTransform: CGAffineTransform?;
    
    /// Event handler: user performed layout on this view
    var performLayout: ((NSSize) -> ())?;
    
    /// Event handler: The bounds of the container have changed
    var boundsChanged: ((ContainerBounds) -> ())?;
    
    /// The minimum size of the scroll area for this view
    var scrollMinimumSize: (Float64, Float64) = (0.0, 0.0);
    
    /// The visibility of the horizontal and vertical scroll bars
    var scrollBarVisibility: (ScrollBarVisibility, ScrollBarVisibility) = (ScrollBarVisibility.Never, ScrollBarVisibility.Never);
    
    /// Triggers the click event for this view
    func triggerClick() {
        bubble_up_event(source: self, event_handler: { (container) in
            if let onClick = container.onClick {
                return onClick();
            } else {
                return false;
            }
        });
    }
    
    ///
    /// Computes the container bounds for this view
    ///
    func getContainerBounds() -> ContainerBounds {
        // Get the bounds
        let viewport        = bounds;
        var visible         = visibleRect;
        
        // For the container bounds, the viewport is considered to be aligned at 0,0
        visible.origin.x    -= viewport.origin.x;
        visible.origin.y    -= viewport.origin.y;
        
        return ContainerBounds(visibleRect: visible, totalSize: viewport.size);
    }

    /// Triggers the bounds changed event for this view
    func triggerBoundsChanged() {
        boundsChanged?(getContainerBounds());
    }
    
    /// Sets the text label for this view
    func setTextLabel(label: String) {
    }
    
    /// Sets the font size for this view
    func setFontSize(points: Float64) {
    }
    
    /// Sets the foreground colour of the control
    func setForegroundColor(color: NSColor) {
    }
    
    /// Sets the font weight for this view
    func setFontWeight(weight: Float64) {
    }
    
    /// Sets the text alignment for this view
    func setTextAlignment(alignment: NSTextAlignment) {
    }
    
    /// Sets part of the state of this control
    func setState(selector: ViewStateSelector, toProperty: FloProperty) {
        
    }
}
