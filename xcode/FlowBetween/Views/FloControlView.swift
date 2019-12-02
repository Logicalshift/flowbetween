//
//  FloControlView.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 03/02/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

///
/// Container view containing a Cocoa control
///
class FloControlView: NSView, FloContainerView, NSTextFieldDelegate {
    /// The control that is displayed in this view
    let _control: NSControl

    /// The font to display the control in
    var _font: NSFont

    /// The foreground colour to display the control text in
    var _color: NSColor?

    /// The alignment of the text in this control
    var _alignment: NSTextAlignment = NSTextAlignment.left

    /// The text in this control
    var _label: String = ""

    /// True if the control is being edited
    var _editing: Bool = false

    required init(frame frameRect: NSRect, control: NSControl) {
        _control    = control
        _font       = NSFontManager.shared.font(withFamily: "Lato", traits: NSFontTraitMask(), weight: 5, size: 13.0)!
        _color      = nil

        super.init(frame: frameRect)

        wantsLayer = true

        _control.target = self
        _control.action = #selector(FloControlView.controlAction)
        _control.frame = bounds
        addSubview(_control)
    }

    required init?(coder decoder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    /// Assuming the control is a text field, centers it vertically
    func centerVerticallyAsTextField() {
        let bounds      = self.bounds
        let text        = _control.attributedStringValue
        let height      = text.size().height

        let center      = bounds.origin.y + bounds.size.height/2.0
        let top         = center - height/2.0

        _control.frame  = NSRect(origin: CGPoint(x: bounds.origin.x, y: top), size: CGSize(width: bounds.size.width, height: height))
    }

    /// The control action has been triggered
    @objc func controlAction() {
        let newValue: PropertyValue

        // Determine the value of the control
        if let slider = _control as? NSSlider {
            newValue = PropertyValue.Float(slider.doubleValue)
        } else if let checkbox = _control as? NSButton {
            newValue = PropertyValue.Bool(checkbox.state == NSControl.StateValue.on)
        } else if let text = _control as? NSTextField {
            newValue = PropertyValue.String(text.stringValue)
        } else {
            newValue = PropertyValue.Nothing
        }

        // If the control is being edited then the event to fire is the 'edit' event, otherwise it's the 'set' event
        var isBeingEdited = false

        if self.window?.currentEvent?.type == .leftMouseDragged {
            _editing        = true
            isBeingEdited   = true
        } else if self.window?.currentEvent?.type == .leftMouseUp {
            _editing        = false
            isBeingEdited   = false
        } else if _editing {
            isBeingEdited   = true
        }

        // Fire the edit or set events as appropriate
        if isBeingEdited {
            onEditValue?(newValue)
        } else {
            onSetValue?(newValue)
        }
    }

    /// If the control is a text field, centers it vertically
    func centerVerticallyIfTextField() {
        if _control is NSTextField {
            centerVerticallyAsTextField()
        }
    }

    /// Updates the frame size of this view
    override func setFrameSize(_ newSize: NSSize) {
        super.setFrameSize(newSize)
        _control.frame = bounds

        centerVerticallyIfTextField()
    }

    /// Adds a subview to this container view
    func addContainerSubview(_ subview: NSView) {
        // Control views cannot have subviews (not supported in Cocoa's model)

        if let containerView = subview as? FloContainerView {
            if let text = containerView.viewState.text {
                // However, we should mirror any labels assigned to the subview
                weak var this = self

                text.trackValue { labelValue in
                    if case .String(let stringValue) = labelValue {
                        this?.setTextLabel(label: stringValue)
                    }
                }
            }
        }
    }

    /// Sets the layer displayed for the canvas
    func setCanvasLayer(_ layer: CALayer) {
        // Control views cannot have layers
    }

    /// The size of the layout area for this view
    var layoutSize : NSSize {
        return self.bounds.size
    }

    /// Stores the general state of this view
    var viewState : ViewState = ViewState()

    /// The FloView that owns this container view (should be a weak reference)
    weak var floView: FloView?

    /// Returns this view as an NSView
    var asView : NSView { return self }

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
        let _ = onClick?()
    }

    ///
    /// Computes the container bounds for this view
    ///
    func getContainerBounds() -> ContainerBounds {
        // Get the bounds
        let viewport        = bounds
        var visible         = visibleRect

        // For the container bounds, the viewport is considered to be aligned at 0,0
        visible.origin.x    -= viewport.origin.x
        visible.origin.y    -= viewport.origin.y

        return ContainerBounds(visibleRect: visible, totalSize: viewport.size)
    }

    /// Triggers the bounds changed event for this view
    func triggerBoundsChanged() {
        boundsChanged?(getContainerBounds())
    }

    /// Sets the text label for this view
    func setTextLabel(label: String) {
        _label                              = label
        _control.attributedStringValue      = attributedLabel

        centerVerticallyIfTextField()
    }

    /// The label with attributes applied
    var attributedLabel: NSAttributedString {
        let paragraphStyle = NSParagraphStyle.default.mutableCopy() as! NSMutableParagraphStyle
        paragraphStyle.alignment = _alignment

        return NSAttributedString(string: _label,
                                    attributes: [NSAttributedString.Key.font: _font,
                                                NSAttributedString.Key.foregroundColor: _color ?? NSColor.white,
                                                NSAttributedString.Key.paragraphStyle: paragraphStyle])
    }

    /// Sets the foreground colour of the control
    func setForegroundColor(color: NSColor) {
        _color = color
        _control.attributedStringValue = attributedLabel
    }

    /// Sets the font size for this view
    func setFontSize(points: Float64) {
        let existingFont    = _font
        let newFont         = NSFontManager.shared.convert(existingFont, toSize: CGFloat(points))

        _font               = newFont

        _control.attributedStringValue = attributedLabel

        centerVerticallyIfTextField()
    }

    ///
    /// Converts a weight from a value like 100, 200, 400, etc to a font manager weight (0-15)
    ///
    func convertWeight(_ weight: Float64) -> Int {
        if weight <= 150.0 {
            return 1
        } else if weight <= 450.0 {
            return 5
        } else if weight <= 750.0 {
            return 7
        } else {
            return 10
        }
    }

    /// Sets the font weight for this view
    func setFontWeight(weight: Float64) {
        let existingFont        = _control.font!
        let fontManagerWeight   = convertWeight(weight)
        let family              = existingFont.familyName!
        let size                = existingFont.pointSize
        let traits              = NSFontTraitMask()

        let newFont             = NSFontManager.shared.font(withFamily: family, traits: traits, weight: fontManagerWeight, size: size) ?? _font

        _font                   = newFont

        _control.attributedStringValue = attributedLabel

        centerVerticallyIfTextField()
    }

    /// Sets the text alignment for this view
    func setTextAlignment(alignment: NSTextAlignment) {
        _alignment = alignment

        _control.attributedStringValue = attributedLabel
    }

    /// If this control's focus priority is higher than the currently focused view, move focus here
    func focusIfNeeded() {
        if let priorityProperty = viewState.focusPriority {
            // Need to decide if we should steal focus from whatever has focus already
            var stealFocus              = false
            let priority                = priorityProperty.value.toDouble(default: 0.0)
            let currentlyFocusedView    = NSView.focusView

            if let currentlyFocusedView = currentlyFocusedView {
                // If the currently focused view has higher priority than us or isn't a FloView then don't steal focus
                if let focusedFloView = FloView.nearestTo(currentlyFocusedView) {
                    let currentPriority = focusedFloView.viewState.focusPriority?.value.toDouble(default: 0.0) ?? 0.0

                    if currentPriority < priority {
                        // We're higher priority than the current view
                        stealFocus = true
                    }
                } else {
                    // There's a focused view in a different part of the UI
                    stealFocus = false
                }
            } else {
                // Should steal focus if there is no focused view
                stealFocus = true
            }

            // Focus the control if we should steal focus
            if stealFocus {
                window?.makeFirstResponder(_control)
            }
        }
    }

    /// When the control moves between windows, it might need to get focus
    override func viewDidMoveToWindow() {
        super.viewDidMoveToWindow()
        focusIfNeeded()
    }

    /// Sets part of the state of this control
    func setState(selector: ViewStateSelector, toProperty: FloProperty) {
        weak var this = self

        // Store the state in the backing state (so the event stays registered)
        viewState.retainProperty(selector: selector, property: toProperty)

        // Track this property
        switch (selector) {
        case .Enabled:
            toProperty.trackValue { enabled in this?._control.isEnabled = enabled.toBool(default: true) }
            break

        case .Value:
            toProperty.trackValue { value in
                switch (value) {
                case .Float(let floatVal):
                    this?._control.doubleValue = Double(floatVal)
                case .Int(let intValue):
                    this?._control.intValue = Int32(intValue)
                case .String(let stringValue):
                    this?._control.stringValue = stringValue
                default:
                    break
                }
            }

        case .RangeLower:
            toProperty.trackValue { value in
                if let slider = this?._control as? NSSlider {
                    slider.minValue = value.toDouble(default: 0.0)
                    this?.viewState.value?.notifyChange()
                }
            }

        case .RangeHigher:
            toProperty.trackValue { value in
                if let slider = this?._control as? NSSlider {
                    slider.maxValue = value.toDouble(default: 1.0)
                    this?.viewState.value?.notifyChange()
                }
            }

        case .FocusPriority:
            toProperty.trackValue { value in this?.focusIfNeeded() }

        case .Selected, .Badged, .LayoutX, .LayoutY:
            break
        }
    }

    /// Delegate method: control began editing text
    func controlTextDidBeginEditing(_ obj: Notification) {
        _editing = true
    }

    /// Delegate method: control finished editing text
    func controlTextDidEndEditing(_ obj: Notification) {
        _editing = false
        onSetValue?(PropertyValue.String(_control.stringValue))
    }

    /// Delegate method: text changed
    func controlTextDidChange(_ obj: Notification) {
        if _editing {
            onEditValue?(PropertyValue.String(_control.stringValue))
        } else {
            onSetValue?(PropertyValue.String(_control.stringValue))
        }
    }
}
