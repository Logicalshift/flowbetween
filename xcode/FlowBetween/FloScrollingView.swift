//
//  FloScrollingView.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 06/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

public class FloScrollingView : NSScrollView, FloContainerView {
    public required init?(coder: NSCoder) {
        super.init(coder: coder)

        self.documentView = FloEmptyView.init(frame: NSRect(x: 0, y: 0, width: 4000, height: 4000));

        self.wantsLayer             = true;
        self.hasHorizontalScroller  = true;
        self.hasVerticalScroller    = true;
        self.autohidesScrollers     = true;
    }
    
    required public override init(frame frameRect: NSRect) {
        super.init(frame: frameRect);

        self.documentView = FloEmptyView.init(frame: NSRect(x: 0, y: 0, width: 4000, height: 4000));

        self.wantsLayer             = true;
        self.hasHorizontalScroller  = true;
        self.hasVerticalScroller    = true;
        self.autohidesScrollers     = true;
    }
    
    override public var isOpaque: Bool { get { return false } }

    ///
    /// Adds a subview to this container
    ///
    func addContainerSubview(_ subview: NSView) {
        self.documentView!.addSubview(subview);
    }

    ///
    /// Containers cause the layout algorithm to run when they are resized
    ///
    override public func setFrameSize(_ newSize: NSSize) {
        super.setFrameSize(newSize);
        
        // Perform normal layout
        self.performLayout?();
        
        // Any subviews that are not themselves FloContainers are sized to fill this view
        for subview in self.documentView!.subviews {
            if (subview as? FloContainerView) == nil {
                subview.frame = NSRect(origin: CGPoint(x: 0, y: 0), size: newSize);
            }
        }
    }

    /// Returns this view as an NSView
    var asView : NSView { get { return self; } };
    
    /// Event handler: user clicked in the view
    var onClick: (() -> Bool)?;
    
    /// Event handler: user performed layout on this view
    var performLayout: (() -> ())?;
    
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
}
