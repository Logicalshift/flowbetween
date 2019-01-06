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
/// View that contains one or more Flo controls
///
class FloContainer : NSView, FloContainerView {
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
    
    ///
    /// Containers are not opaque
    ///
    override public var isOpaque: Bool { get { return false; } }

    ///
    /// Containers cause the layout algorithm to run when they are resized
    ///
    override public func setFrameSize(_ newSize: NSSize) {
        super.setFrameSize(newSize);
        
        // Perform normal layout
        self.performLayout?();
        
        // Any subviews that are not themselves FloContainers are sized to fill this view
        for subview in subviews {
            if (subview as? FloContainer) == nil {
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
        bubble_up(handler: { (container) in
            if let onClick = container.onClick {
                return onClick();
            } else {
                return false;
            }
        });
    }
    
    ///
    /// Bubbles an event up from this view
    ///
    public func bubble_up(handler: (FloContainer) -> Bool) {
        if handler(self) {
            // Event was handled
            return;
        } else {
            // Bubble up to the superview
            var bubble_to = superview;
            
            while true {
                if let bubble_to_view = bubble_to {
                    // Try this view
                    if let bubble_to = bubble_to_view as? FloContainer {
                        return bubble_to.bubble_up(handler: handler);
                    }

                    // Try the superview
                    bubble_to = bubble_to_view.superview;
                }
            }
        }
    }
}
