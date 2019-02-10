//
//  FloContainerButton.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 09/02/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

///
/// Container view that acts like a button
///
class FloContainerButton : NSView, FloContainerView {
    fileprivate let _backingLayer = FloContainerButtonLayer();
    
    override init(frame frameRect: NSRect) {
        super.init(frame: frameRect);
        
        wantsLayer              = true;
        layer                   = _backingLayer;
        layer?.backgroundColor  = CGColor.clear;
        layer?.isOpaque         = false;
        layer?.setNeedsDisplay();
    }
    
    required init?(coder decoder: NSCoder) {
        super.init(coder: decoder);

        wantsLayer              = true;
        layer                   = _backingLayer;
        layer?.backgroundColor  = CGColor.clear;
        layer?.isOpaque         = false;
        layer?.setNeedsDisplay();
    }
    
    var _trackingArea: NSTrackingArea?;
    
    /// Updates the frame size of this control
    override func setFrameSize(_ newSize: NSSize) {
        super.setFrameSize(newSize);
        triggerBoundsChanged();
    }
    
    /// Updates the tracking area for this view
    override func updateTrackingAreas() {
        if let trackingArea = _trackingArea {
            self.removeTrackingArea(trackingArea);
            _trackingArea = nil;
        }
        
        let trackingArea = NSTrackingArea(rect: bounds,
                                          options: NSTrackingArea.Options.mouseEnteredAndExited.union(NSTrackingArea.Options.activeAlways),
                                          owner: self, userInfo: nil);
        self.addTrackingArea(trackingArea);
        _trackingArea = trackingArea;
    }
    
    /// User has pressed the mouse down in this view
    override func mouseDown(with event: NSEvent) {
        // TODO: track the mouse and make sure it stays within the bounds of the control
        triggerClick();
    }
    
    override func mouseEntered(with event: NSEvent) {
        _backingLayer.highlighted = true;
    }

    override func mouseExited(with event: NSEvent) {
        _backingLayer.highlighted = false;
    }

    /// Adds a subview to this container view
    func addContainerSubview(_ subview: NSView) {
        addSubview(subview);
    }
    
    /// Sets the layer displayed for the canvas
    func setCanvasLayer(_ layer: CALayer) {
        
    }
    
    /// Stores the general state of this view
    var viewState : ViewState = ViewState();
    
    /// The size of the layout area for this view
    var layoutSize : NSSize {
        get {
            return self.bounds.size;
        }
    }
    
    /// Returns this view as an NSView
    var asView : NSView {
        get { return self }
    };
    
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
    
    /// Events handlers when a particular device is used for painting
    var onPaint: [FloPaintDevice: (FloPaintStage, AppPainting) -> ()] = [FloPaintDevice: (FloPaintStage, AppPainting) -> ()]();
    
    /// The affine transform for the canvas layer
    var canvasAffineTransform: CGAffineTransform?;
    
    /// Event handler: user performed layout on this view
    var performLayout: ((NSSize) -> ())?;
    
    /// Event handler: The bounds of the container have changed
    var boundsChanged: ((ContainerBounds) -> ())?;
    
    /// The minimum size of the scroll area for this view
    var scrollMinimumSize: (Float64, Float64) = (0,0);
    
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

    /// Computes the container bounds for this view
    func getContainerBounds() -> ContainerBounds {
        // Get the bounds
        let viewport        = bounds;
        var visible         = visibleRect;
        
        // For the container bounds, the viewport is considered to be aligned at 0,0
        visible.origin.x    -= viewport.origin.x;
        visible.origin.y    -= viewport.origin.y;
        
        return ContainerBounds(visibleRect: visible, totalSize: viewport.size);
    }

    fileprivate var _willChangeBounds = false;
    /// Triggers the bounds changed event for this view
    func triggerBoundsChanged() {
        if !_willChangeBounds {
            _willChangeBounds = true;
            
            RunLoop.current.perform(inModes: [RunLoop.Mode.default, RunLoop.Mode.eventTracking], block: {
                self._willChangeBounds = false;
                
                let bounds = self.getContainerBounds();
                self.boundsChanged?(bounds);
            });
        }
    }
    
    /// Sets the text label for this view
    func setTextLabel(label: String) {
        // Text not supported by this view
    }
    
    /// Sets the font size for this view
    func setFontSize(points: Float64) {
        // Text not supported by this view
    }
    
    /// Sets the foreground colour of the control
    func setForegroundColor(color: NSColor) {
        // Text not supported by this view
    }
    
    /// Sets the font weight for this view
    func setFontWeight(weight: Float64) {
        // Text not supported by this view
    }
    
    /// Sets the text alignment for this view
    func setTextAlignment(alignment: NSTextAlignment) {
        // Text not supported by this view
    }
    
    /// Sets part of the state of this control
    func setState(selector: ViewStateSelector, toProperty: FloProperty) {
        // Store the property in the view state
        viewState.retainProperty(selector: selector, property: toProperty);
        
        weak var this = self;
        
        switch (selector) {
        case .Selected:
            toProperty.trackValue({ newValue in
                this?._backingLayer.selected = newValue.toBool(default: false);
            });
            
        default:
            // Not supported by this view
            break;
        }
    }
}
