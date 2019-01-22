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
    
    /// Returns this view as an NSView
    public var asView: NSView { get { return self; } }
    
    /// Event handler: user clicked in the view
    public var onClick: (() -> Bool)?;
    
    /// Event handler: user performed layout on this view
    public var performLayout: (() -> ())?;

    /// Event handler: user scrolled/resized so that a particular region is visible
    public var onScroll: ((NSRect) -> ())?;

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
        self.performLayout?();
        
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

    /// Sets the layer displayed for the canvas
    func setCanvasLayer(_ layer: CALayer) {
        self.layer!.addSublayer(layer);
    }
    
}
