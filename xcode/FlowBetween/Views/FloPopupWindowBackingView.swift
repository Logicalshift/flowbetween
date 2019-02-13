//
//  FloPopupWindowBackingVIew.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 13/02/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

///
/// The backing view for a popup window
///
class FloPopupWindowBackingView : NSView {
    override func setFrameSize(_ newSize: NSSize) {
        subviews.forEach { subview in
            subview.frame = self.bounds;
        };
    }
    
    /// The popup direction for this view
    var direction: PopupDirection = .Above;
    
    ///
    /// Calulates the required size for this view given a content size and a direction
    ///
    func sizeForContentSize(_ contentSize: NSSize, _ direction: PopupDirection) -> NSSize {
        return contentSize;
    }
}
