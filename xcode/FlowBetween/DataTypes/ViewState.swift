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
    var rangeLower:     FloProperty?;
    var rangeHigher:    FloProperty?;
    var focusPriority:  FloProperty?;
    
    var layout_x:       [FloProperty] = [];
    var layout_y:       [FloProperty] = [];
    
    ///
    /// Removes all layout properties that are being tracked in this view
    ///
    func clearLayoutProperties() {
        layout_x = [];
        layout_y = [];
    }
    
    ///
    /// Stores the property associated with a selector in this view state
    ///
    func retainProperty(selector: ViewStateSelector, property: FloProperty) {
        switch (selector) {
        case .Selected:         selected = property;
        case .Badged:           badged = property;
        case .Enabled:          enabled = property;
        case .Value:            value = property;
        case .RangeLower:       rangeLower = property;
        case .RangeHigher:      rangeHigher = property;
        case .FocusPriority:    focusPriority = property;
        case .LayoutX:          layout_x.append(property);
        case .LayoutY:          layout_y.append(property);
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
