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
class Layout {
    ///
    /// Lays out a single position
    ///
    static func layoutPosition(pos: Position, previous: Double, end: Double, stretch_distance: Double, stretch_total: Double) -> Double {
        switch pos {
        case Position.At(let value):        return Double(value);
        case Position.Offset(let value):    return previous + Double(value);
        case Position.After:                return previous;
        case Position.Start:                return 0.0;
        case Position.End:                  return end;

        case Position.Stretch(let stretch):
            if stretch_total > 0.0 {
                let proportion = Double(stretch) / stretch_total;
                return previous + (stretch_distance*proportion);
            } else {
                return previous;
            }
        }
    }
    
    ///
    /// Updates the stretch total based on a position
    ///
    static func updateStretch(pos: Position, last_stretch: Double) -> Double {
        switch pos {
        case Position.At(_):                return last_stretch;
        case Position.Offset(_):            return last_stretch;
        case Position.After:                return last_stretch;
        case Position.Start:                return last_stretch;
        case Position.End:                  return last_stretch;
            
        case Position.Stretch(let stretch): return last_stretch+Double(stretch);
        }
    }
    
    ///
    /// Lays out the specified view according to the bounds set for its subviews
    ///
    public static func layoutView(view: FloView) {
        let bounds          = view.bounds;
        let max_x           = Double(bounds.width);
        let max_y           = Double(bounds.height);
        var last_x          = 0.0;
        var last_y          = 0.0;
        var stretch_total_x = 0.0;
        var stretch_total_y = 0.0;
        
        // First pass: all stretched views are set to 0 (calibrating the stretch distances)
        for subview in view.subviews {
            // Only FloViews get laid out
            if let subview = subview as? FloView {
                // Get the bounds for this view
                let bounds = subview.floBounds;
                
                // Only need the x2 and y2 positions for the first pass
                let x2 = layoutPosition(pos: bounds.x2, previous: last_x, end: max_x, stretch_distance: 0.0, stretch_total: 0.0);
                let y2 = layoutPosition(pos: bounds.y2, previous: last_y, end: max_y, stretch_distance: 0.0, stretch_total: 0.0);
                
                // Update the stretch totals
                stretch_total_x = updateStretch(pos: bounds.x1, last_stretch: stretch_total_x);
                stretch_total_x = updateStretch(pos: bounds.x2, last_stretch: stretch_total_x);
                stretch_total_y = updateStretch(pos: bounds.y1, last_stretch: stretch_total_y);
                stretch_total_y = updateStretch(pos: bounds.y2, last_stretch: stretch_total_y);

                // Update the last_x and last_y values
                last_x = x2;
                last_y = y2;
            }
        }
        
        // Calculate the stretch distances
        let stretch_distance_x = max_x - last_x;
        let stretch_distance_y = max_y - last_y;
        
        // Reset for the layout pass
        last_x = 0;
        last_y = 0;
        
        // Actually perform the layout
        for subview in view.subviews {
            // Only FloViews get laid out
            if let subview = subview as? FloView {
                // Get the bounds for this view
                let bounds = subview.floBounds;
                
                // Compute the x1, y1, x2, y2 positions
                let x1 = layoutPosition(pos: bounds.x1, previous: last_x, end: max_x, stretch_distance: stretch_distance_x, stretch_total: stretch_total_x);
                let x2 = layoutPosition(pos: bounds.x2, previous: last_x, end: max_x, stretch_distance: stretch_distance_x, stretch_total: stretch_total_x);
                let y1 = layoutPosition(pos: bounds.y1, previous: last_y, end: max_y, stretch_distance: stretch_distance_y, stretch_total: stretch_total_y);
                let y2 = layoutPosition(pos: bounds.y2, previous: last_y, end: max_y, stretch_distance: stretch_distance_y, stretch_total: stretch_total_y);
                
                // Set the new frame for the view
                let frame = NSRect(x: x1, y: max_y-y2, width: x2-x1, height: y2-y1);
                subview.frame = frame;

                // Update the last_x and last_y values
                last_x = x2;
                last_y = y2;
            } else {
                // Other subviews are set to fill the entire control
                subview.frame = NSRect(x: 0, y: 0, width: max_x, height: max_y);
            }
        }
    }
}
