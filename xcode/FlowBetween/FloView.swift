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
    /// The view that this will display
    fileprivate var _view: NSView!;
    
    override init() {
    }
    
    ///
    /// The view that this is managing
    ///
    public var view: NSView! {
        get { return _view; }
    }
    
    ///
    /// Creates an empty view
    ///
    @objc public func setupAsEmpty() {
        // Just a standard NSView
        _view = NSView.init();
        
        // Create core animation views wherever possible
        _view.wantsLayer = true;
    }
    
    ///
    /// Removes this view from its superview
    ///
    @objc public func viewRemoveFromSuperview() {
        _view?.removeFromSuperview();
    }
    
    ///
    /// Adds a subview to this view
    ///
    @objc(viewAddSubView:) public func viewAddSubView(subview: FloView!) {
        if let subview = subview._view {
            _view?.addSubview(subview);
        }
    }
}
