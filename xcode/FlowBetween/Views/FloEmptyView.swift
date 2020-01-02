//
//  FloEmptyView.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 06/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

extension NSRect {
    init(size: CGSize) {
        self.init(origin: .zero, size: size)
    }
}

///
/// View that contains one or more Flo controls
///
class FloEmptyView : NSView, FloContainerView {
    var _canvasLayer: CALayer?
    var _labelView: FloControlView?

    required override init(frame frameRect: NSRect) {
        super.init(frame: frameRect)

        wantsLayer = true
    }

    required init?(coder decoder: NSCoder) {
        super.init(coder: decoder)

        wantsLayer = true
    }

    /// The size of the layout area for this view
    var layoutSize : NSSize {
        return bounds.size
    }

    /// Stores the general state of this view
    var viewState : ViewState = ViewState()

    /// The FloView that owns this container view (should be a weak reference)
    public weak var floView: FloView?

    /// Returns this view as an NSView
    public var asView: NSView { return self }

    /// Event handler: user clicked in the view
    public var onClick: (() -> Bool)?

    /// Event handler: user performed layout on this view
    public var performLayout: ((NSSize) -> ())?

    /// Event handler: user scrolled/resized so that a particular region is visible
    public var onScroll: ((NSRect) -> ())?

    /// Event handler: value has changed
    public var onEditValue: ((PropertyValue) -> ())?

    /// Event handler: value has been set
    public var onSetValue: ((PropertyValue) -> ())?

    /// Event handler: control has obtained keyboard focus
    public var onFocused: (() -> ())?

    /// Event handler: user has dragged this control
    public var onDrag: ((DragAction, CGPoint, CGPoint) -> ())?

    /// Event handlers when particular devices are used for painting actions
    public var onPaint: [FloPaintDevice: (FloPaintStage, AppPainting) -> ()] = [FloPaintDevice: (FloPaintStage, AppPainting) -> ()]()

    var _canvasAffineTransform: CGAffineTransform?
    var _invertCanvasTransform: CGAffineTransform = .identity

    /// The affine transform for the canvas layer
    var canvasAffineTransform: CGAffineTransform?
    {
        get { return _canvasAffineTransform }
        set(value) {
            _canvasAffineTransform = value

            if let value = value {
                _invertCanvasTransform = value.inverted()
            } else {
                _invertCanvasTransform = .identity
            }
        }
    }

    /// Event handler: The bounds of the container have changed
    public var boundsChanged: ((ContainerBounds) -> ())? {
        didSet {
            triggerBoundsChanged()
        }
    }

    /// The minimum size of the scroll area for this view
    public var scrollMinimumSize: (Float64, Float64) = (0, 0)

    /// The visibility of the horizontal and vertical scroll bars
    public var scrollBarVisibility: (ScrollBarVisibility, ScrollBarVisibility) = (ScrollBarVisibility.Never, ScrollBarVisibility.Never)

    ///
    /// Containers are not opaque
    ///
    override public var isOpaque: Bool { return false }

    ///
    /// Containers use a flipped coordinate system
    ///
    override var isFlipped: Bool { return true }

    ///
    /// Adds a subview to this container
    ///
    func addContainerSubview(_ subview: NSView) {
        self.addSubview(subview)
    }

    ///
    /// Containers cause the layout algorithm to run when they are resized
    ///
    override public func setFrameSize(_ newSize: NSSize) {
        super.setFrameSize(newSize)

        // Perform normal layout
        triggerBoundsChanged()
        performLayout?(bounds.size)

        // Any subviews that are not themselves FloContainers are sized to fill this view
        _labelView?.frame = NSRect(size: newSize)
        for subview in subviews {
            if (subview as? FloContainerView) == nil {
                subview.frame = NSRect(size: newSize)
            }
        }
    }

    ///
    /// User released the mouse (while it was not captured)
    ///
    override public func mouseUp(with event: NSEvent) {
        if onClick != nil || onDrag != nil {
            if event.modifierFlags == [] && event.buttonNumber == 0 {
                triggerClick()
            }
        }
    }

    ///
    /// Triggers the click event
    ///
    public func triggerClick() {
        bubbleUpEvent(source: self) { $0.onClick?() ?? false }
    }

    ///
    /// Sets the layer displayed for the canvas
    ///
    func setCanvasLayer(_ layer: CALayer) {
        _canvasLayer = layer
        self.layer!.addSublayer(layer)
    }

    ///
    /// Computes the container bounds for this view
    ///
    func getContainerBounds() -> ContainerBounds {
        // Get the bounds
        let viewport        = bounds
        var visible         = visibleRect

        // For small enough containers, always display the whole area
        let area = visible.width * visible.height
        if area < 800*600 {
            visible = viewport
        }

        // For the container bounds, the viewport is considered to be aligned at 0,0
        visible.origin.x    -= viewport.origin.x
        visible.origin.y    -= viewport.origin.y

        return ContainerBounds(visibleRect: visible, totalSize: viewport.size)
    }

    var _willChangeBounds: Bool = false
    ///
    /// Triggers the bounds changed event for this view
    ///
    func triggerBoundsChanged() {
        if !_willChangeBounds {
            _willChangeBounds = true

            RunLoop.current.perform(inModes: [.default, .eventTracking]) {
                self._willChangeBounds = false

                let bounds = self.getContainerBounds()
                self.boundsChanged?(bounds)
            }
        }
    }

    ///
    /// The label view for this empty view
    ///
    var labelView: FloControlView {
        if _labelView == nil {
            let label   = NSTextField(labelWithString: "")
            label.font  = NSFontManager.shared.font(withFamily: "Lato", traits: [], weight: 5, size: 13.0)

            _labelView = FloControlView(frame: bounds, control: label)
            addSubview(_labelView!)
        }

        return _labelView!
    }

    /// Sets the text label for this view
    func setTextLabel(label: String) {
        labelView.setTextLabel(label: label)
    }

    /// Sets the font size for this view
    func setFontSize(points: Float64) {
        labelView.setFontSize(points: points)
    }

    /// Sets the font weight for this view
    func setFontWeight(weight: Float64) {
        labelView.setFontWeight(weight: weight)
    }

    /// Sets the text alignment for this view
    func setTextAlignment(alignment: NSTextAlignment) {
        labelView.setTextAlignment(alignment: alignment)
    }

    /// Sets the foreground colour of the control
    func setForegroundColor(color: NSColor) {
        labelView.setForegroundColor(color: color)
    }

    /// Sets part of the state of this control
    func setState(selector: ViewStateSelector, toProperty: FloProperty) {
        viewState.retainProperty(selector: selector, property: toProperty)
    }

    ///
    /// Returns the paint device that a particular event represents
    ///
    func paintDeviceForEvent(_ event: NSEvent) -> FloPaintDevice? {
        if event.subtype == NSEvent.EventSubtype.tabletPoint
            || event.subtype == NSEvent.EventSubtype.tabletProximity {
            // Is a tablet event
            if getCurrentTabletPointingDevice(fromEvent: event) == .eraser {
                // Eraser pointing device
                return FloPaintDevice.Eraser
            } else {
                // Pen pointing device
                return FloPaintDevice.Pen
            }
        } else {
            // Is a general mouse event
            if event.type == NSEvent.EventType.leftMouseDown {
                return FloPaintDevice.MouseLeft
            } else if event.type == NSEvent.EventType.rightMouseDown {
                return FloPaintDevice.MouseRight
            } else if event.type == NSEvent.EventType.otherMouseDown && event.buttonNumber == 2{
                return FloPaintDevice.MouseMiddle
            } else {
                return nil
            }
        }
    }

    ///
    /// Generates the AppPainting data from an NSEvent
    ///
    func createAppPainting(event: NSEvent) -> AppPainting {
        // Work out the location of the event
        let bounds              = self.bounds
        let locationInWindow    = event.locationInWindow
        let locationInView      = self.convert(locationInWindow, from: nil)
        var locationInCanvas    = locationInView

        locationInCanvas.y      = bounds.height - locationInCanvas.y

        if let canvasLayer = _canvasLayer {
            let layerFrame = canvasLayer.frame

            locationInCanvas.x -= layerFrame.origin.x
            locationInCanvas.y += layerFrame.origin.y

            // Need to invert the coordinates
            locationInCanvas.y  = bounds.size.height - locationInCanvas.y
        }

        locationInCanvas = locationInCanvas.applying(_invertCanvasTransform)

        return AppPainting(
            pointer_id: 0,
            position_x: Double(locationInCanvas.x),
            position_y: Double(locationInCanvas.y),
            pressure:   Double(event.pressure),
            tilt_x:     0.0,
            tilt_y:     0.0
        )
    }

    ///
    /// Returns the event mask to use for tracking events following a particular mouse action
    ///
    func eventMaskForInitialMouseEvent(event: NSEvent) -> NSEvent.EventTypeMask {
        var eventMask = NSEvent.EventTypeMask.leftMouseDragged.union(NSEvent.EventTypeMask.leftMouseUp)
        switch (event.type) {
        case .leftMouseDown:
            break
        case .rightMouseDown:
            eventMask = NSEvent.EventTypeMask.rightMouseDragged.union(NSEvent.EventTypeMask.rightMouseUp)
            break
        case .otherMouseDown:
            eventMask = NSEvent.EventTypeMask.otherMouseDragged.union(NSEvent.EventTypeMask.otherMouseUp)
            break
        default:
            break
        }

        return eventMask
    }

    ///
    /// Relays paint events for the specified device. The initial event should be a 'mouse down'
    /// type event that initiated the paint actions.
    ///
    func paint(with device: FloPaintDevice, initialEvent: NSEvent, paintAction: (FloPaintStage, AppPainting) -> ()) {
        // Send the paint start event
        paintAction(FloPaintStage.Start, createAppPainting(event: initialEvent))

        // Event mask depends on the initial event
        let eventMask = eventMaskForInitialMouseEvent(event: initialEvent)

        // Track events until the mouse is released
        var done = false
        while (!done) {
            // Fetch the next event that might mach our device
            let nextEvent = window?.nextEvent(matching: eventMask, until: Date.distantFuture, inMode: .eventTracking, dequeue: true)

            if let nextEvent = nextEvent {
                // Check that the event is for the same device as started the paint action
                if nextEvent.pointingDeviceID != initialEvent.pointingDeviceID { continue }

                // Check if it's a finish event
                let isFinished = nextEvent.type == .leftMouseUp || nextEvent.type == .rightMouseUp || nextEvent.type == .otherMouseUp

                // Send the painting action
                autoreleasepool {
                    if !isFinished {
                        paintAction(FloPaintStage.Continue, createAppPainting(event: nextEvent))
                    } else {
                        paintAction(FloPaintStage.Finish, createAppPainting(event: nextEvent))
                        done = true
                    }
                }
            }
        }
    }

    ///
    /// Trying to drag a view that can be dragged (and optionally producing a click event instead)
    ///
    func tryDrag(onDrag: (DragAction, CGPoint, CGPoint) -> (), initialEvent: NSEvent) {
        // The initial pos is used for the 'from' coordinates for the drag
        // We use an origin point so if the view moves (or is removed) during the drag we continue to generate points consistent with the original drag origin
        var origin              = self.convert(CGPoint(x: 0, y: 0), to: nil)
        let size                = self.bounds.size
        origin.y                -= size.height
        let initialPosInWindow  = initialEvent.locationInWindow
        let initialPos          = CGPoint(x: initialPosInWindow.x - origin.x, y: size.height - (initialPosInWindow.y - origin.y))

        // If there's an onClick handler, the user needs to drag a certain minimum distance away before we start the 'real' drag instead of a click
        var dragging    = false
        let minDistance = CGFloat(8.0)

        // Mouse events immediately produce a drag if there's no onClick handler
        if self.onClick == nil {
            dragging = true
            onDrag(.Start, initialPos, initialPos)
        }

        // Track the mouse until the user releases the mouse button
        let window      = self.window
        let eventMask   = eventMaskForInitialMouseEvent(event: initialEvent)
        var done        = false
        while (!done) {
            autoreleasepool {
                // Fetch the next mouse event
                let nextEvent = window?.nextEvent(matching: eventMask, until: Date.distantFuture, inMode: .eventTracking, dequeue: true)

                if let nextEvent = nextEvent {
                    // Position relative to this view
                    let nextPosInWindow = nextEvent.locationInWindow
                    let nextPos         = CGPoint(x: nextPosInWindow.x-origin.x, y: size.height - (nextPosInWindow.y-origin.y))

                    // Lifting whichever button we're tracking counts as a finish event
                    let isFinished  = nextEvent.type == .leftMouseUp || nextEvent.type == .rightMouseUp || nextEvent.type == .otherMouseUp

                    // Start dragging if necessary
                    if !dragging {
                        let offset      = CGPoint(x: initialPos.x-nextPos.x, y: initialPos.y-nextPos.y)
                        let distance    = (offset.x*offset.x + offset.y*offset.y).squareRoot()

                        if distance > minDistance {
                            dragging = true
                            onDrag(.Start, initialPos, initialPos)
                        }
                    }

                    // Send the next drag event
                    if dragging {
                        onDrag(.Continue, initialPos, nextPos)
                    }

                    // Finish dragging if necessary
                    if isFinished {
                        done = true

                        if dragging {
                            // Finish the drag
                            onDrag(.Finish, initialPos, nextPos)
                        } else {
                            // Drag never started
                            triggerClick()
                        }
                    }
                } else {
                    // No event?
                    done = true

                    if dragging {
                        onDrag(.Cancel, initialPos, initialPos)
                    }
                }
            }
        }
    }

    ///
    /// Left mouse is down
    ///
    override func mouseDown(with event: NSEvent) {
        // Start painting if there's a paint action attached to the device this event is for
        if let device = paintDeviceForEvent(event) {
            if let paintAction = onPaint[device] {
                paint(with: device, initialEvent: event, paintAction: paintAction)
                return
            }
        }

        // Start tracking a drag if there's a drag action attached to this view
        if let onDrag = onDrag {
            tryDrag(onDrag: onDrag, initialEvent: event)
        }

        // Send the event up the view stack if this view can't handle it
        if onDrag == nil && onClick == nil {
            superview?.mouseDown(with: event)
            return
        }

    }

    ///
    /// Right mouse is down
    ///
    override func rightMouseDown(with event: NSEvent) {
        // Start painting if there's a paint action attached to the device this event is for
        if let device = paintDeviceForEvent(event) {
            if let paintAction = onPaint[device] {
                paint(with: device, initialEvent: event, paintAction: paintAction)
                return
            }
        }
    }

    ///
    /// Other mouse is down (generally middle button)
    ///
    override func otherMouseDown(with event: NSEvent) {
        // Start painting if there's a paint action attached to the device this event is for
        if let device = paintDeviceForEvent(event) {
            if let paintAction = onPaint[device] {
                paint(with: device, initialEvent: event, paintAction: paintAction)
                return
            }
        }
    }
}
