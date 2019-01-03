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
        
    }
    
    ///
    /// Creates an empty view
    ///
    public func setupAsEmpty() {
        
    }
    
    ///
    /// Removes this view from its superview
    ///
    public func viewRemoveFromSuperview() {
        NSLog("Remove from superview");
    }
    
    ///
    /// Adds a subview to this view
    ///
    public func viewAddSubView(subview: FloView) {
        NSLog("Add SubView");
    }
}
