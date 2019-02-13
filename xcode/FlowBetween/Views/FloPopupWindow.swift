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
    /// The backing view for this window
    let _backingView = FloPopupWindowBackingView();
    
    required init() {
        super.init(contentRect: NSRect.init(),
                   styleMask: NSWindow.StyleMask.borderless,
                   backing: NSWindow.BackingStoreType.buffered,
                   defer: true);
        
        isOpaque    = true;
        level       = .floating;
        
        contentView = _backingView;
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
        let centerPoint         = CGPoint(x: relativeToScreen.origin.x + relativeToScreen.width/2.0, y: relativeToScreen.origin.y + relativeToScreen.height/2.0);
        
        let contentSize         = _backingView.sizeForContentSize(popupContentSize, direction);
        let centered            = CGPoint(x: centerPoint.x - (contentSize.width/2.0), y: centerPoint.y - contentSize.height/2.0);
        var position            = centered;
        
        switch (direction) {
        case .Above:            position = CGPoint(x: centered.x, y: relativeToScreen.maxY + contentSize.height + offset); break;
        case .Below:            position = CGPoint(x: centered.x, y: relativeToScreen.minY - contentSize.height - offset); break;
        case .Left:             position = CGPoint(x: relativeToScreen.minX - contentSize.width - offset, y: centered.y); break;
        case .Right:            position = CGPoint(x: relativeToScreen.maxX + contentSize.width + offset, y: centered.y); break;
        case .OnTop:            position = centered; break;
        case .WindowCentered:   position = centered; break;
        case .WindowTop:        position = centered; break;
        }
        
        // Update the content size
        setContentSize(contentSize);
        
        // Move into position
        setFrameOrigin(position);
    }
}
