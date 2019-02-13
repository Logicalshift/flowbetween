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
class FloPopupWindow : NSWindow {
    required init() {
        super.init(contentRect: NSRect.init(),
                   styleMask: NSWindow.StyleMask.borderless,
                   backing: NSWindow.BackingStoreType.buffered,
                   defer: true);
        
        isOpaque    = true;
        level       = .floating;
        
        contentView = FloPopupWindowBackingView();
        contentView!.wantsLayer = true;
        contentView!.addSubview(popupContentView);
    }
    
    /// Sets the view that this window is drawn relative to
    func setParentView(view: NSView) {
        _parentView = view;
    }
    
    /// The view that owns this popup window
    fileprivate weak var _parentView: NSView?;
    
    /// The view that contains the main content for this window
    let popupContentView: NSView = NSView();
    
    /// The direction that this popup window should display in
    var direction: PopupDirection = .Below;
    
    /// The offset in the direction that the popup window is oriented
    var offset: CGFloat = 0.0;
    
    /// The content size of this popup window
    var popupContentSize: NSSize = NSSize();
    
    ///
    /// Updates the position of this popup view
    ///
    func updatePosition() {
        // Work out where to place the window
        let screenOrigin        = _parentView?.window?.frame.origin ?? CGPoint();
        let parentBounds        = _parentView?.bounds ?? NSRect();
        let relativeToWindow    = _parentView?.convert(parentBounds, to: nil) ?? NSRect();
        let relativeToScreen    = CGRect(origin: CGPoint(x: relativeToWindow.origin.x+screenOrigin.x, y: relativeToWindow.origin.y+screenOrigin.y), size: relativeToWindow.size);
        
        var contentSize         = popupContentSize;
        var position            = CGPoint(x: relativeToScreen.origin.x - (contentSize.width/2.0), y: relativeToScreen.origin.y);
        
        // Update the content size
        setContentSize(contentSize);
        
        // Move into position
        setFrameOrigin(position);
    }
}
