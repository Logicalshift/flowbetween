//
//  FloPopupWindow.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 13/02/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

extension CGRect {
    var center: CGPoint {
        return CGPoint(x: midX, y: midY)
    }
}

///
/// Window used to display a popup
///
class FloPopupWindow : NSWindow {
    /// The backing view for this window
    let _backingView = FloPopupWindowBackingView()

    required init() {
        super.init(contentRect: .zero,
                   styleMask: .borderless,
                   backing: .buffered,
                   defer: true)

        isOpaque        = false
        level           = .floating
        backgroundColor = NSColor.clear

        _backingView.wantsLayer = true
        _backingView.addSubview(popupContentView)
        contentView = _backingView
    }

    /// Sets the view that this window is drawn relative to
    func setParentView(view: NSView) {
        _parentView = view
    }

    /// The view that owns this popup window
    fileprivate weak var _parentView: NSView?
    var parentView: NSView? { return _parentView }

    /// The view that contains the main content for this window
    let popupContentView: NSView = NSView()

    /// The direction that this popup window should display in
    var direction: PopupDirection = .Below

    /// The offset in the direction that the popup window is oriented
    var offset: CGFloat = 0.0

    /// The content size of this popup window
    var popupContentSize: NSSize = NSSize()

    ///
    /// Updates the position of this popup view
    ///
    func updatePosition() {
        // Update the content view
        _backingView.direction = direction

        // Work out where to place the window
        let screenOrigin        = _parentView?.window?.frame.origin ?? .zero
        let parentBounds        = _parentView?.bounds ?? .zero
        let relativeToWindow    = _parentView?.convert(parentBounds, to: nil) ?? .zero
        let relativeToScreen    = CGRect(origin: CGPoint(x: relativeToWindow.origin.x+screenOrigin.x, y: relativeToWindow.origin.y+screenOrigin.y), size: relativeToWindow.size)
        let centerPoint         = relativeToScreen.center

        let contentSize         = _backingView.sizeForContentSize(popupContentSize, direction)
        let centered            = CGPoint(x: centerPoint.x - (contentSize.width/2.0), y: centerPoint.y - contentSize.height/2.0)
        var position            = centered

        switch (direction) {
        case .Above:            position = CGPoint(x: centered.x, y: relativeToScreen.maxY + contentSize.height + offset)
        case .Below:            position = CGPoint(x: centered.x, y: relativeToScreen.minY - contentSize.height - offset)
        case .Left:             position = CGPoint(x: relativeToScreen.minX - contentSize.width - offset, y: centered.y)
        case .Right:            position = CGPoint(x: relativeToScreen.maxX + contentSize.width + offset, y: centered.y)
        case .OnTop, .WindowCentered, .WindowTop:            position = centered
        }

        // Update the content size
        setContentSize(contentSize)

        // Move into position
        setFrameOrigin(position)
    }
}
