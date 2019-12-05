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
    var text:           FloProperty?
    var popupOpen:      FloProperty?

    var selected:       FloProperty?
    var badged:         FloProperty?
    var enabled:        FloProperty?
    var value:          FloProperty?
    var rangeLower:     FloProperty?
    var rangeHigher:    FloProperty?
    var focusPriority:  FloProperty?

    var layoutX:        [FloProperty]   = []
    var layoutY:        [FloProperty]   = []

    var fixedAxis:      FixedAxis       = FixedAxis.None
    let subviewIndex:   FloProperty     = FloProperty(withInt: 0)
    let isFirst:        FloProperty     = FloProperty(withBool: false)
    let isLast:         FloProperty     = FloProperty(withBool: false)
    var zIndex:         CGFloat?
    var classes:        [String]        = []

    ///
    /// Removes all layout properties that are being tracked in this view
    ///
    func clearLayoutProperties() {
        layoutX = []
        layoutY = []
    }

    ///
    /// Stores the property associated with a selector in this view state
    ///
    func retainProperty(selector: ViewStateSelector, property: FloProperty) {
        switch (selector) {
        case .Selected:         selected = property
        case .Badged:           badged = property
        case .Enabled:          enabled = property
        case .Value:            value = property
        case .RangeLower:       rangeLower = property
        case .RangeHigher:      rangeHigher = property
        case .FocusPriority:    focusPriority = property
        case .LayoutX:          layoutX.append(property)
        case .LayoutY:          layoutY.append(property)
        }
    }
}

///
/// Selector that indicates part of the view state
///
enum ViewStateSelector {
    case Selected
    case Badged
    case Enabled
    case Value
    case RangeLower
    case RangeHigher
    case FocusPriority
    case LayoutX
    case LayoutY
}
