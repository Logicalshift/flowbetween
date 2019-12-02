//
//  Layout.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 03/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

///
/// Performs layout on a view according to how its size changes
///
class FloLayout {
    ///
    /// Lays out a single position
    ///
    static func layoutPosition(pos: Position, previous: Double, end: Double, stretch_distance: Double, stretch_total: Double) -> Double {
        switch pos {
        case .At(let value):                return Double(value)
        case .Offset(let value):            return previous + Double(value)
        case .Floating(let value, _):       return Double(value)
        case .After:                        return previous
        case .Start:                        return 0.0
        case .End:                          return end

        case .Stretch(let stretch):
            if stretch_total > 0.0 {
                let proportion = Double(stretch) / stretch_total
                return previous + (stretch_distance*proportion)
            } else {
                return previous
            }
        }
    }

    ///
    /// Updates the stretch total based on a position
    ///
    static func updateStretch(pos: Position, last_stretch: Double) -> Double {
        switch pos {
        case .At:
            return last_stretch
        case .Offset:
            return last_stretch
        case .Floating:
            return last_stretch
        case .After:
            return last_stretch
        case .Start:
            return last_stretch
        case .End:
            return last_stretch
        case .Stretch(let stretch):
            return last_stretch+Double(stretch)
        }
    }

    ///
    /// Lays out the specified view according to the bounds set for its subviews
    ///
    public static func layoutView(view: FloView, size: NSSize, state: ViewState) {
        let bounds          = NSRect(size: size)
        let padding         = view.floPadding ?? Padding(left: 0, top: 0, right: 0, bottom: 0)
        let max_x           = Double(bounds.width) - padding.left - padding.right
        let max_y           = Double(bounds.height) - padding.top - padding.bottom
        var last_x          = 0.0
        var last_y          = 0.0
        var stretch_total_x = 0.0
        var stretch_total_y = 0.0

        // Remove any existing floating properties from the view state
        state.clearLayoutProperties()

        // First pass: all stretched views are set to 0 (calibrating the stretch distances)
        for subview in view.layoutSubviews {
            // Get the bounds for this view
            let bounds = subview.floBounds

            // Only need the x2 and y2 positions for the first pass
            let x1 = layoutPosition(pos: bounds.x1, previous: last_x, end: max_x, stretch_distance: 0.0, stretch_total: 0.0)
            let x2 = layoutPosition(pos: bounds.x2, previous: x1, end: max_x, stretch_distance: 0.0, stretch_total: 0.0)
            let y1 = layoutPosition(pos: bounds.y1, previous: last_y, end: max_y, stretch_distance: 0.0, stretch_total: 0.0)
            let y2 = layoutPosition(pos: bounds.y2, previous: y1, end: max_y, stretch_distance: 0.0, stretch_total: 0.0)

            // Update the stretch totals
            stretch_total_x = updateStretch(pos: bounds.x1, last_stretch: stretch_total_x)
            stretch_total_x = updateStretch(pos: bounds.x2, last_stretch: stretch_total_x)
            stretch_total_y = updateStretch(pos: bounds.y1, last_stretch: stretch_total_y)
            stretch_total_y = updateStretch(pos: bounds.y2, last_stretch: stretch_total_y)

            // Update the last_x and last_y values
            last_x = x2
            last_y = y2
        }

        // Calculate the stretch distances
        let stretch_distance_x = max_x - last_x
        let stretch_distance_y = max_y - last_y

        // Reset for the layout pass
        last_x = 0
        last_y = 0

        // Actually perform the layout
        for subview in view.layoutSubviews {
            // Get the bounds for this view
            let bounds = subview.floBounds

            // Compute the x1, y1, x2, y2 positions
            let x1 = layoutPosition(pos: bounds.x1, previous: last_x, end: max_x, stretch_distance: stretch_distance_x, stretch_total: stretch_total_x)
            let x2 = layoutPosition(pos: bounds.x2, previous: x1, end: max_x, stretch_distance: stretch_distance_x, stretch_total: stretch_total_x)
            let y1 = layoutPosition(pos: bounds.y1, previous: last_y, end: max_y, stretch_distance: stretch_distance_y, stretch_total: stretch_total_y)
            let y2 = layoutPosition(pos: bounds.y2, previous: y1, end: max_y, stretch_distance: stretch_distance_y, stretch_total: stretch_total_y)

            // Set the new frame for the view (TODO: floating both in x1 and y1)
            // TODO: stop any old tracking if we're re-doing the layout
            let frame = NSRect(x: x1 + padding.left, y: y1 + padding.top, width: x2-x1, height: y2-y1)
            subview.view.frame = frame

            // Float in the x and the y directions
            var float_x = 0.0
            var float_y = 0.0

            if case let Position.Floating(_, prop) = bounds.x1 {
                // Update the position whenever the floating property changes
                state.retainProperty(selector: ViewStateSelector.LayoutX, property: prop) // TODO: retain all properties here

                prop.trackValue { floating_offset_property in
                    var floating_offset = 0.0
                    if case let PropertyValue.Float(val) = floating_offset_property {
                        floating_offset = val
                    } else if case let PropertyValue.Int(val) = floating_offset_property {
                        floating_offset = Double(val)
                    }

                    float_x = floating_offset

                    let frame = NSRect(x: x1 + float_x + padding.left, y: y1 + float_y + padding.top, width: x2-x1, height: y2-y1)
                    subview.view.frame = frame
                }
            }

            if case let Position.Floating(_, prop) = bounds.y1 {
                // Update the position whenever the floating property changes
                state.retainProperty(selector: ViewStateSelector.LayoutY, property: prop) // TODO: retain all properties here

                prop.trackValue { floating_offset_property in
                    var floating_offset = 0.0
                    if case let PropertyValue.Float(val) = floating_offset_property {
                        floating_offset = val
                    } else if case let PropertyValue.Int(val) = floating_offset_property {
                        floating_offset = Double(val)
                    }

                    float_y = floating_offset

                    let frame = NSRect(x: x1 + float_x + padding.left, y: y1 + float_y + padding.top, width: x2-x1, height: y2-y1)
                    subview.view.frame = frame
                }
            }

            // Update the last_x and last_y values
            last_x = x2
            last_y = y2
        }
    }
}
