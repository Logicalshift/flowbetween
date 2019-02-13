//
//  FloPopupWindow.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 13/02/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

///
/// Window used to display a popup
///
class FloPopupWindow : NSPanel {
    required init() {
        super.init(contentRect: NSRect.init(),
                   styleMask: NSWindow.StyleMask.borderless.union(NSWindow.StyleMask.hudWindow),
                   backing: NSWindow.BackingStoreType.buffered,
                   defer: true);
        
        isOpaque = true;
    }
    
    ///
    /// Sets the popup content size for this window
    ///
    func setPopupContentSize(_ size: NSSize) {
        setContentSize(size);
    }
}
