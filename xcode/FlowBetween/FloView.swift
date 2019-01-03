//
//  FloView.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 03/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Foundation
import Cocoa

///
/// Class used to manage a view in FlowBetween
///
public class FloView : NSObject {
    override init() {
        NSLog("New view");
    }
    
    ///
    /// Creates an empty view
    ///
    @objc public func setupAsEmpty() {
        NSLog("Setup as empty");
    }
    
    ///
    /// Removes this view from its superview
    ///
    @objc public func viewRemoveFromSuperview() {
        NSLog("Remove from superview");
    }
    
    ///
    /// Adds a subview to this view
    ///
    @objc(viewAddSubView:) public func viewAddSubView(subview: FloView) {
        NSLog("Add SubView");
    }
}
