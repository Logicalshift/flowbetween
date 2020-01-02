//
//  FloRotorView.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 16/02/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

///
/// View that can be used to pick a value by rotating the view like a slider
///
class FloRotorView : FloEmptyView {
    /// The value representing 0 degrees for this rotor view
    fileprivate var _lowerRange: CGFloat = 0.0

    /// The value representing 360 degrees for this rotor view
    fileprivate var _upperRange: CGFloat = 1.0

    /// The current value for this rotor view
    fileprivate var _value: CGFloat = 0.0

    /// True if the rotor is being dragged at the moment
    fileprivate var _dragging = false

    ///
    /// Updates the value of this view
    ///
    func updateValue() {
        // Set the angle of the view
        let ratio                       = (_value-_lowerRange)/(_upperRange-_lowerRange)
        let angle                       = CGFloat.pi*2.0 * ratio

        let bounds                      = self.bounds
        let center                      = CGPoint(x: bounds.width/2.0, y: bounds.height/2.0)

        var transform                   = CATransform3DIdentity
        transform                       = CATransform3DTranslate(transform, center.x, center.y, 0.0)
        transform                       = CATransform3DRotate(transform, angle, 0.0, 0.0, 1.0)
        transform                       = CATransform3DTranslate(transform, -center.x, -center.y, 0.0)

        layer?.transform                = transform
    }

    ///
    /// Superview changed
    ///
    override func viewDidMoveToSuperview() {
        if let valueProperty = viewState.value {
            _value = CGFloat(valueProperty.value.toDouble(default: 0.0))
        }

        updateValue()
    }

    ///
    /// Need to change the value transform when the view bounds change
    ///
    override func setFrameSize(_ newSize: NSSize) {
        super.setFrameSize(newSize)
        updateValue()
    }

    ///
    /// Sets the state for this view
    ///
    override func setState(selector: ViewStateSelector, toProperty: FloProperty) {
        weak var this = self

        switch (selector) {
        case .RangeLower:
            viewState.retainProperty(selector: selector, property: toProperty)
            toProperty.trackValue { newValue in
                this?._lowerRange = CGFloat(newValue.toDouble(default: 0.0))
                this?.updateValue()
            }
            break

        case .RangeHigher:
            viewState.retainProperty(selector: selector, property: toProperty)
            toProperty.trackValue { newValue in
                this?._upperRange = CGFloat(newValue.toDouble(default: 0.0))
                this?.updateValue()
            }
            break

        case .Value:
            viewState.retainProperty(selector: selector, property: toProperty)
            toProperty.trackValue { newValue in
                if !(this?._dragging ?? false) {
                    this?._value = CGFloat(newValue.toDouble(default: 0.0))
                    this?.updateValue()
                }
            }
            break

        default:
            super.setState(selector: selector, toProperty: toProperty)
        }
    }

    ///
    /// Computes the angle in radians for a position relative to this view
    ///
    func angleForPoint(_ point: CGPoint) -> CGFloat {
        // Assume that the node is a circle around its center
        let radius = bounds.width/2.0

        // Recompute the point relative to the center of the rotor
        let x = point.x-bounds.width/2.0
        let y = -point.y-bounds.height/2.0

        if ((x*x + y*y) < (radius*radius)) {
            // If the point is within the main radius, then the angle is just the angle relative to the center
            return atan2(y, x)
        } else {
            // Really want to project a line onto the circle, then make the
            // extra angle be the distance from the rotor. This has a
            // similar effect but isn't quite as accurate.
            let angle           = atan2(y, x)
            let circumference   = CGFloat.pi*2*radius
            var extra_distance  = -x
            if (x < -radius) {
                extra_distance -= radius
            } else if (x > radius) {
                extra_distance += radius
            } else {
                extra_distance = 0
            }

            return angle + ((extra_distance/circumference) * CGFloat.pi*2)
        }
    }

    ///
    /// Dragging the rotor changes its value
    ///
    override func mouseDown(with event: NSEvent) {
        // Store the initial drag parameters
        let initialEvent    = event
        let origin          = convert(CGPoint(x: 0, y: 0), to: nil)
        let initialPos      = initialEvent.locationInWindow
        let initialAngle    = angleForPoint(CGPoint(x: initialPos.x-origin.x, y: initialPos.y-origin.y))
        let initialValue    = _value

        // Flag that we're dragging so value updates are ignored
        _dragging           = true

        // Track the mouse until the user releases the mouse button
        let window          = self.window
        let eventMask       = eventMaskForInitialMouseEvent(event: initialEvent)
        var done            = false
        while (!done) {
            autoreleasepool {
                // Fetch the next mouse event
                let nextEvent = window?.nextEvent(matching: eventMask, until: Date.distantFuture, inMode: .eventTracking, dequeue: true)

                if let nextEvent = nextEvent {
                    // Position relative to this view
                    let nextPosInWindow = nextEvent.locationInWindow
                    let nextPos         = CGPoint(x: nextPosInWindow.x-origin.x, y: nextPosInWindow.y-origin.y)
                    let nextAngle       = angleForPoint(nextPos)

                    // Lifting whichever button we're tracking counts as a finish event
                    let isFinished  = nextEvent.type == .leftMouseUp || nextEvent.type == .rightMouseUp || nextEvent.type == .otherMouseUp

                    // Compute the difference in value
                    let angleDiff       = nextAngle-initialAngle
                    let angleRatio      = angleDiff / (2*CGFloat.pi)
                    let valueDiff       = (_upperRange-_lowerRange)*angleRatio
                    let nextValue       = initialValue + valueDiff
                    let nextValueMod    = (nextValue-_lowerRange).truncatingRemainder(dividingBy: _upperRange-_lowerRange) + _lowerRange

                    // Update the value
                    _value = nextValueMod
                    updateValue()

                    // Send a value update
                    if !isFinished {
                        self.onEditValue?(PropertyValue.Float(Double(_value)))
                    } else {
                        done = true
                        self.onSetValue?(PropertyValue.Float(Double(_value)))
                    }
                } else {
                    // No event?
                    done = true
                    self.onSetValue?(PropertyValue.Float(Double(_value)))
                }
            }
        }

        // Done dragging
        _dragging = false
    }
}
