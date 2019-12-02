//
//  FloPopupView.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 13/02/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

///
/// Flo view that tracks a popup window
///
class FloPopupView : NSView, FloContainerView, FloContainerPopup {
    /// The window that displays the popup
    fileprivate var _popupWindow: FloPopupWindow = FloPopupWindow()

    /// The window that currently has the popup window set as a child window
    fileprivate weak var _owningWindow: NSWindow?

    override init(frame frameRect: NSRect) {
        super.init(frame: frameRect)

        _popupWindow.setParentView(view: self)
    }

    required init?(coder decoder: NSCoder) {
        super.init(coder: decoder)

        _popupWindow.setParentView(view: self)
    }

    deinit {
        _popupWindow.orderOut(self)
        _owningWindow?.removeChildWindow(_popupWindow)
        _owningWindow = nil
    }

    override var isOpaque: Bool {
        return false
    }

    override func viewWillMove(toWindow newWindow: NSWindow?) {
        _owningWindow?.removeChildWindow(_popupWindow)
        _owningWindow = nil

        if _popupWindow.isVisible {
            newWindow?.addChildWindow(_popupWindow, ordered: .above)
            _owningWindow = newWindow
        }
    }

    override func viewDidMoveToWindow() {
        _popupWindow.updatePosition()
    }

    override func setFrameSize(_ newSize: NSSize) {
        super.setFrameSize(newSize)
        _popupWindow.updatePosition()
    }

    /// Adds a subview to this container view
    func addContainerSubview(_ subview: NSView) {
        _popupWindow.popupContentView.addSubview(subview)
    }

    /// Sets the layer displayed for the canvas
    func setCanvasLayer(_ layer: CALayer) {
        _popupWindow.popupContentView.wantsLayer = true
        _popupWindow.popupContentView.layer = layer
    }

    /// Stores the general state of this view
    var viewState : ViewState = ViewState()

    /// The size of the layout area for this view
    var layoutSize : NSSize {
        // TODO: size of the view in the popup window
        return _popupWindow.popupContentSize
    }

    /// The FloView that owns this container view (should be a weak reference)
    weak var floView: FloView?

    /// Returns this view as an NSView
    var asView : NSView {
        return self
    }

    /// Event handler: user clicked in the view
    var onClick: (() -> Bool)?

    /// Event handler: user scrolled/resized so that a particular region is visible
    var onScroll: ((NSRect) -> ())?

    /// Event handler: value has changed
    var onEditValue: ((PropertyValue) -> ())?

    /// Event handler: value has been set
    var onSetValue: ((PropertyValue) -> ())?

    /// Event handler: control has obtained keyboard focus
    var onFocused: (() -> ())?

    /// Event handler: user has dragged this control
    var onDrag: ((DragAction, CGPoint, CGPoint) -> ())?

    /// Events handlers when a particular device is used for painting
    var onPaint: [FloPaintDevice: (FloPaintStage, AppPainting) -> ()] = [FloPaintDevice: (FloPaintStage, AppPainting) -> ()]()

    /// The affine transform for the canvas layer
    var canvasAffineTransform: CGAffineTransform?

    /// Event handler: user performed layout on this view
    var performLayout: ((NSSize) -> ())?

    /// Event handler: The bounds of the container have changed
    var boundsChanged: ((ContainerBounds) -> ())?

    /// The minimum size of the scroll area for this view
    var scrollMinimumSize: (Float64, Float64) = (0.0, 0.0)

    /// The visibility of the horizontal and vertical scroll bars
    var scrollBarVisibility: (ScrollBarVisibility, ScrollBarVisibility) = (ScrollBarVisibility.Never, ScrollBarVisibility.Never)

    /// Triggers the click event for this view
    func triggerClick() {
        bubbleUpEvent(source: self) { $0.onClick?() ?? false }
    }

    ///
    /// Computes the container bounds for this view
    ///
    func getContainerBounds() -> ContainerBounds {
        // Get the bounds
        let viewport        = CGRect(size: _popupWindow.popupContentSize)
        var visible         = viewport

        // For the container bounds, the viewport is considered to be aligned at 0,0
        visible.origin.x    -= viewport.origin.x
        visible.origin.y    -= viewport.origin.y

        return ContainerBounds(visibleRect: visible, totalSize: viewport.size)
    }

    /// Triggers the bounds changed event for this view
    func triggerBoundsChanged() {
        boundsChanged?(getContainerBounds())
        _popupWindow.updatePosition()
    }

    /// Sets the text label for this view
    func setTextLabel(label: String) {
    }

    /// Sets the font size for this view
    func setFontSize(points: Float64) {
    }

    /// Sets the foreground colour of the control
    func setForegroundColor(color: NSColor) {
    }

    /// Sets the font weight for this view
    func setFontWeight(weight: Float64) {
    }

    /// Sets the text alignment for this view
    func setTextAlignment(alignment: NSTextAlignment) {
    }

    /// Sets part of the state of this control
    func setState(selector: ViewStateSelector, toProperty: FloProperty) {

    }

    /// Sets whether or not the popup view is open
    func setPopupOpen(_ isOpen: Bool) {
        if isOpen {
            if _owningWindow == nil {
                _owningWindow = window
                window?.addChildWindow(_popupWindow, ordered: .above)
            }

            _popupWindow.updatePosition()
            _popupWindow.orderFront(self)
        } else {
            _popupWindow.orderOut(self)
        }
    }

    /// Sets the direction that the popup window appears in relative to the parent window
    func setPopupDirection(_ direction: PopupDirection) {
        _popupWindow.direction = direction
        _popupWindow.updatePosition()
    }

    /// Sets the sisze of the popup
    func setPopupSize(width: CGFloat, height: CGFloat) {
        _popupWindow.popupContentSize = NSSize(width: width, height: height)
        triggerBoundsChanged()
    }

    /// Sets the offset of the popup in the popup direction
    func setPopupOffset(_ offset: CGFloat) {
        _popupWindow.offset = offset
        _popupWindow.updatePosition()
    }
}
