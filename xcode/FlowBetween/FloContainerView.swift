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
