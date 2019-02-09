//
//  ViewState.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 09/02/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Foundation

///
/// Storage for the state of a FloContainerView
///
class ViewState {
    var selected:       FloProperty?;
    var badged:         FloProperty?;
    var enabled:        FloProperty?;
    var value:          FloProperty?;
    var range_lower:    FloProperty?;
    var range_higher:   FloProperty?;
    var focus_priority: FloProperty?;
    
    var layout_x:       FloProperty?;
    var layout_y:       FloProperty?;
    
    ///
    /// Stores the property associated with a selector in this view state
    ///
    func retainProperty(selector: ViewStateSelector, property: FloProperty) {
        switch (selector) {
        case .Selected:         selected = property;
        case .Badged:           badged = property;
        case .Enabled:          enabled = property;
        case .Value:            value = property;
        case .RangeLower:       range_lower = property;
        case .RangeHigher:      range_higher = property;
        case .FocusPriority:    focus_priority = property;
        case .LayoutX:          layout_x = property;
        case .LayoutY:          layout_y = property;
        }
    }
}

///
/// Selector that indicates part of the view state
///
enum ViewStateSelector {
    case Selected;
    case Badged;
    case Enabled;
    case Value;
    case RangeLower;
    case RangeHigher;
    case FocusPriority;
    case LayoutX;
    case LayoutY;
}
