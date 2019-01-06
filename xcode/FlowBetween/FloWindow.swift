//
//  FloWindow.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 02/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Foundation
import Cocoa

///
/// Represents a window created by FlowBetween
///
public class FloWindow : NSObject, NSWindowDelegate {
    ///
    /// The window itself
    ///
    fileprivate var _window: NSWindow;
    
    ///
    /// The root view, if the window has one
    ///
    fileprivate var _rootView: FloView?;
    
    override init() {
        // Create the window
        let styleMask: NSWindow.StyleMask = [.resizable, .closable, .titled];
        
        _window = NSWindow.init(
            contentRect:    NSRect(x: 100, y: 100, width: 1600, height: 960),
            styleMask:      styleMask,
            backing:        NSWindow.BackingStoreType.buffered,
            defer:          true);
        
        super.init();
        
        _window.title = "FlowBetween session";
        _window.delegate = self;
    }
    
    ///
    /// Opens the window
    ///
    @objc public func windowOpen() {
        _window.makeKeyAndOrderFront(self);
    }
    
    ///
    /// Sets the root view of this window
    ///
    @objc(windowSetRootView:) public func windowSetRootView(view: FloView!) {
        _rootView               = view;
        _window.contentView     = view.view;
    }
}
