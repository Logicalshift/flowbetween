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

        //self.contentView.setFrameSize(NSSize(width: 4000, height: 4000));
    }
    
    required public override init(frame frameRect: NSRect) {
        super.init(frame: frameRect);
        
        //self.contentView.setFrameSize(NSSize(width: 4000, height: 4000));
    }
    
    override public var isOpaque: Bool { get { return false } }

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
