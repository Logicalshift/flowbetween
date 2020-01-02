//
//  FloContainerButton.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 09/02/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

///
/// Container view that acts like a button
///
class FloContainerButton : NSView, FloContainerView {
    /// The layer that the button is drawn on
    fileprivate let _backingLayer = FloContainerButtonLayer()

    /// Layer that displays the badge for this button
    fileprivate var _badgeLayer: CALayer?

    override init(frame frameRect: NSRect) {
        self.viewState = ViewState();
        
        super.init(frame: frameRect)

        weak var this           = self
        wantsLayer              = true
        layer                   = _backingLayer
        layer?.backgroundColor  = CGColor.clear
        layer?.isOpaque         = false
        layer?.setNeedsDisplay()

        viewState.isFirst.trackValue({ isFirst in this?._backingLayer.isFirst = isFirst.toBool(default: false) })
        viewState.isLast.trackValue({ isLast in this?._backingLayer.isLast = isLast.toBool(default: false) })
    }

    required init?(coder decoder: NSCoder) {
        self.viewState = ViewState();

        super.init(coder: decoder)

        weak var this           = self
        wantsLayer              = true
        layer                   = _backingLayer
        layer?.backgroundColor  = CGColor.clear
        layer?.isOpaque         = false
        layer?.setNeedsDisplay()

        viewState.isFirst.trackValue({ isFirst in this?._backingLayer.isFirst = isFirst.toBool(default: false) })
        viewState.isLast.trackValue({ isLast in this?._backingLayer.isLast = isLast.toBool(default: false) })
    }

    var _trackingArea: NSTrackingArea?

    /// Updates the frame size of this control
    override func setFrameSize(_ newSize: NSSize) {
        super.setFrameSize(newSize)
        triggerBoundsChanged()
    }

    /// Updates the tracking area for this view
    override func updateTrackingAreas() {
        if let trackingArea = _trackingArea {
            self.removeTrackingArea(trackingArea)
            _trackingArea = nil
        }

        let trackingArea = NSTrackingArea(rect: bounds,
                                          options: [.mouseEnteredAndExited, .activeAlways],
                                          owner: self, userInfo: nil)
        self.addTrackingArea(trackingArea)
        _trackingArea = trackingArea
    }

    override func viewDidMoveToSuperview() {
        super.viewDidMoveToSuperview()

        _backingLayer.classes = classNamesForView(self)
    }

    override func viewDidMoveToWindow() {
        super.viewDidMoveToWindow()

        _backingLayer.classes = classNamesForView(self)
    }

    /// User has pressed the mouse down in this view
    override func mouseDown(with event: NSEvent) {
        // TODO: track the mouse and make sure it stays within the bounds of the control
        if _backingLayer.enabled {
            // Trigger the click only if the button is actually enabled
            triggerClick()
        }
    }

    override func mouseEntered(with event: NSEvent) {
        _backingLayer.highlighted = true
    }

    override func mouseExited(with event: NSEvent) {
        _backingLayer.highlighted = false
    }

    /// Adds a subview to this container view
    func addContainerSubview(_ subview: NSView) {
        addSubview(subview)
    }

    /// Sets the layer displayed for the canvas
    func setCanvasLayer(_ layer: CALayer) {

    }

    /// Stores the general state of this view
    var viewState : ViewState {
        didSet {
            weak var this = self
            viewState.isFirst.trackValue({ isFirst in this?._backingLayer.isFirst = isFirst.toBool(default: false) })
            viewState.isLast.trackValue({ isLast in this?._backingLayer.isLast = isLast.toBool(default: false) })
        }
    }

    /// The size of the layout area for this view
    var layoutSize : NSSize {
        return self.bounds.size
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
    var scrollMinimumSize: (Float64, Float64) = (0,0)

    /// The visibility of the horizontal and vertical scroll bars
    var scrollBarVisibility: (ScrollBarVisibility, ScrollBarVisibility) = (ScrollBarVisibility.Never, ScrollBarVisibility.Never)

    /// Triggers the click event for this view
    func triggerClick() {
        bubbleUpEvent(source: self, event_handler: { (container) in
            if let onClick = container.onClick {
                return onClick()
            } else {
                return false
            }
        })
    }

    /// Computes the container bounds for this view
    func getContainerBounds() -> ContainerBounds {
        // Get the bounds
        let viewport        = bounds
        var visible         = visibleRect

        // For the container bounds, the viewport is considered to be aligned at 0,0
        visible.origin.x    -= viewport.origin.x
        visible.origin.y    -= viewport.origin.y

        return ContainerBounds(visibleRect: visible, totalSize: viewport.size)
    }

    fileprivate var _willChangeBounds = false
    /// Triggers the bounds changed event for this view
    func triggerBoundsChanged() {
        if !_willChangeBounds {
            _willChangeBounds = true

            RunLoop.current.perform(inModes: [.default, .eventTracking], block: {
                self._willChangeBounds = false

                let bounds = self.getContainerBounds()
                self.boundsChanged?(bounds)

                if let screen = self.window?.screen {
                    self._backingLayer.contentsScale = screen.backingScaleFactor
                }
            })
        }
    }

    /// Sets the text label for this view
    func setTextLabel(label: String) {
        // Text not supported by this view
    }

    /// Sets the font size for this view
    func setFontSize(points: Float64) {
        // Text not supported by this view
    }

    /// Sets the foreground colour of the control
    func setForegroundColor(color: NSColor) {
        // Text not supported by this view
    }

    /// Sets the font weight for this view
    func setFontWeight(weight: Float64) {
        // Text not supported by this view
    }

    /// Sets the text alignment for this view
    func setTextAlignment(alignment: NSTextAlignment) {
        // Text not supported by this view
    }

    /// Sets part of the state of this control
    func setState(selector: ViewStateSelector, toProperty: FloProperty) {
        // Store the property in the view state
        viewState.retainProperty(selector: selector, property: toProperty)

        weak var this = self

        switch (selector) {
        case .Selected:
            toProperty.trackValue({ newValue in
                this?._backingLayer.selected = newValue.toBool(default: false)
            })

        case .Badged:
            toProperty.trackValue({ newValue in
                // Decide whether or not to show the badge
                let showBadge               = newValue.toBool(default: false)
                this?._backingLayer.badged  = showBadge

                // Display the badge layer if needed
                if let this = this {
                    // Create the badge layer if it doesn't exist
                    if this._badgeLayer == nil {
                        this._badgeLayer = FloBadgeLayer()
                    }

                    // Starts out removed
                    this._badgeLayer!.removeFromSuperlayer()

                    // Show/hide the badge
                    if showBadge {
                        this.layer?.addSublayer(this._badgeLayer!)
                        this._badgeLayer?.setNeedsDisplay()
                    }
                }
            })

        case .Enabled:
            toProperty.trackValue({ newValue in
                this?._backingLayer.enabled = newValue.toBool(default: true)

                if this?._backingLayer.enabled ?? true {
                    self.layer!.filters = []
                } else {
                    let greyFilter = CIFilter(name: "CIPhotoEffectNoir")
                    self.layer!.filters = [greyFilter as Any]
                }
            })

        default:
            // Not supported by this view
            break
        }
    }
}
