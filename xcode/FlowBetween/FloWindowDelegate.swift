//
//  FloWindow.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 02/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Foundation
import Cocoa

class FloWindow : NSWindow {
    deinit {
        NSLog("FloWindow deinit");
    }
}

///
/// Represents a window created by FlowBetween
///
public class FloWindowDelegate : NSObject, NSWindowDelegate {
    ///
    /// The window itself
    ///
    fileprivate var _window: FloWindow;
    
    ///
    /// The root view, if the window has one
    ///
    fileprivate var _rootView: FloView?;
    
    ///
    /// The session that this window is for
    ///
    fileprivate weak var _session : NSObject?;
    
    ///
    /// The session ID for this window
    ///
    fileprivate var _sessionId: UInt64;
    
    @objc required init(_ session: FloControl!) {
        // Create the window
        let styleMask: NSWindow.StyleMask = [.resizable, .closable, .titled];
        
        _window = FloWindow(
            contentRect:    NSRect(x: 100, y: 100, width: 1600, height: 960),
            styleMask:      styleMask,
            backing:        NSWindow.BackingStoreType.buffered,
            defer:          true);
        _session    = session;
        _sessionId  = session.sessionId();
        
        super.init();
        
        _window.title = "FlowBetween session";
        _window.delegate = self;
    }
    
    deinit {
        NSLog("FloWindowDelegate deinit");
    }
    
    ///
    /// The window will close
    ///
    @objc public func windowWillClose(_ notification: Notification) {
        // Remove the session from the main app delegate
        if let delegate = NSApp.delegate as? FloAppDelegate {
            delegate.finishSessionWithId(_sessionId);
        }
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
    @objc public func windowSetRootView(_ view: FloView!) {
        _rootView               = view;
        _window.contentView     = view.view;
    }
    
    func triggerAllBoundsChanged() {
        // Notify all the FloViews that the bounds have changed
        var remainingViews = [_window.contentView!];
        
        while let nextView = remainingViews.popLast() {
            // Trigger the bounds changed event on any container views
            if let containerView = nextView as? FloContainerView {
                containerView.triggerBoundsChanged();
            }
            
            // Process the entire view tree
            remainingViews.append(contentsOf: nextView.subviews);
        }
    }
    
    ///
    /// The backing properties (colour scheme, resolution) of the window was changed
    ///
    @objc public func windowDidChangeBackingProperties(_ notification: Notification) {
        triggerAllBoundsChanged();
    }
    
    ///
    /// Callback when a tick has occurred
    ///
    @objc func tick() {
        if let session = _session {
            // The Rust obj-c crate doesn't provide a way to generate the linker symbols necessary to call FloControl directly
            session.perform(#selector(tick));
        }
    }
    
    ///
    /// Request for a tick event to be generated
    ///
    @objc public func requestTick() {
        // Cocoa doesn't really have a way to request an animation frame other than by delaying. We'll use a delay indicating 120fps here
        RunLoop.main.perform {
            self.perform(#selector(self.tick),
                         with: nil,
                         afterDelay: TimeInterval.init(1.0 / 120.0),
                         inModes: [RunLoop.Mode.default, RunLoop.Mode.eventTracking, RunLoop.Mode.modalPanel]);
        }
    }
}
