//
//  FloEmptyView.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 06/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

///
/// View that contains one or more Flo controls
///
class FloEmptyView : NSView, FloContainerView {
    required override init(frame frameRect: NSRect) {
        super.init(frame: frameRect);
        
        wantsLayer = true;
    }
    
    required init?(coder decoder: NSCoder) {
        super.init(coder: decoder);
        
        wantsLayer = true;
    }
    
    /// The size of the layout area for this view
    var layoutSize : NSSize {
        get {
            return bounds.size;
        }
    };

    /// Returns this view as an NSView
    public var asView: NSView { get { return self; } }
    
    /// Event handler: user clicked in the view
    public var onClick: (() -> Bool)?;
    
    /// Event handler: user performed layout on this view
    public var performLayout: ((NSSize) -> ())?;

    /// Event handler: user scrolled/resized so that a particular region is visible
    public var onScroll: ((NSRect) -> ())?;

    /// Event handlers when particular devices are used for painting actions
    public var onPaint: [FloPaintDevice: (FloPaintStage, AppPainting) -> ()] = [FloPaintDevice: (FloPaintStage, AppPainting) -> ()]();

    var _boundsChanged: ((ContainerBounds) -> ())?;
    /// Event handler: The bounds of the container have changed
    public var boundsChanged: ((ContainerBounds) -> ())?
    {
        get { return _boundsChanged; }
        set(value) {
            _boundsChanged = value;
            triggerBoundsChanged();
        }
    }

    /// The minimum size of the scroll area for this view
    public var scrollMinimumSize: (Float64, Float64) = (0, 0);
    
    /// The visibility of the horizontal and vertical scroll bars
    public var scrollBarVisibility: (ScrollBarVisibility, ScrollBarVisibility) = (ScrollBarVisibility.Never, ScrollBarVisibility.Never);

    ///
    /// Containers are not opaque
    ///
    override public var isOpaque: Bool { get { return false; } }
    
    ///
    /// Containers use a flipped coordinate system
    ///
    override var isFlipped: Bool { get { return true; } }
    
    ///
    /// Adds a subview to this container
    ///
    func addContainerSubview(_ subview: NSView) {
        self.addSubview(subview);
    }
    
    ///
    /// Containers cause the layout algorithm to run when they are resized
    ///
    override public func setFrameSize(_ newSize: NSSize) {
        super.setFrameSize(newSize);
        
        // Perform normal layout
        triggerBoundsChanged();
        performLayout?(bounds.size);
        
        // Any subviews that are not themselves FloContainers are sized to fill this view
        for subview in subviews {
            if (subview as? FloContainerView) == nil {
                subview.frame = NSRect(origin: CGPoint(x: 0, y: 0), size: newSize);
            }
        }
    }
    
    ///
    /// User released the mouse (while it was not captured)
    ///
    override public func mouseUp(with event: NSEvent) {
        if event.modifierFlags == NSEvent.ModifierFlags() && event.buttonNumber == 0 {
            triggerClick();
        }
    }
    
    ///
    /// Triggers the click event
    ///
    public func triggerClick() {
        bubble_up_event(source: self, event_handler: { (container) in
            if let onClick = container.onClick {
                return onClick();
            } else {
                return false;
            }
        });
    }

    ///
    /// Sets the layer displayed for the canvas
    ///
    func setCanvasLayer(_ layer: CALayer) {
        self.layer!.addSublayer(layer);
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

    var _willChangeBounds: Bool = false;
    ///
    /// Triggers the bounds changed event for this view
    ///
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
}
