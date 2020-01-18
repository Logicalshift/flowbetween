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
}

///
/// Represents a window created by FlowBetween
///
public class FloWindowDelegate : NSObject, NSWindowDelegate {
    ///
    /// The window itself
    ///
    fileprivate var _window: FloWindow

    ///
    /// The root view, if the window has one
    ///
    fileprivate var _rootView: FloView?

    ///
    /// The session that this window is for
    ///
    fileprivate weak var _session : NSObject?

    ///
    /// The session ID for this window
    ///
    fileprivate var _sessionId: UInt64

    @objc required init(_ session: FloControl!) {
        // Create the window
        let styleMask: NSWindow.StyleMask = [.resizable, .closable, .titled]

        _window = FloWindow(
            contentRect:    NSRect(x: 100, y: 100, width: 1600, height: 960),
            styleMask:      styleMask,
            backing:        .buffered,
            defer:          true)
        _session    = session
        _sessionId  = session.sessionId()

        // ??????? Cocoa bug ???????
        //
        // Can't tie this to FlowBetween's code at all. Whenever windowWillClose is called, the window
        // is released, regardless of the strong reference stored by this class. It's then released
        // again when this class is freed due to that strong reference, causing a crash.
        //
        // If this class doesn't stop the session after windowWillClose (which means nothing other than
        // the Swift side is freeing windows) then the window is still freed up on close. This appears
        // to be down to Cocoa.
        //
        // This adds an extra retain to the window so that it's only freed once. Not sure why a similar
        // issue does not happen for the popup window. Rust side does manual reference counting but it
        // never finds out about the window directly and doesn't double-free anything else so there's
        // nowhere to add an extra retain on that side.
        //
        // ... reproduced this with a really simple test case, this is a real bug in AppKit
        let buggyRetain = Unmanaged.passUnretained(_window).retain()
        let _ = buggyRetain

        super.init()

        _window.title = "FlowBetween session"
        _window.delegate = self
    }

    ///
    /// The window will close
    ///
    @objc public func windowWillClose(_ notification: Notification) {
        // Remove the session from the main app delegate
        if let delegate = NSApp.delegate as? FloAppDelegate {
            delegate.finishSessionWithId(_sessionId)
        }

        // Tidy up the window views (in case buggyRetain fails to work)
        _window.contentView = nil
    }

    ///
    /// Opens the window
    ///
    @objc public func windowOpen() {
        _window.makeKeyAndOrderFront(self)
    }

    ///
    /// Sets the root view of this window
    ///
    @objc public func windowSetRootView(_ view: FloView!) {
        _rootView               = view
        _window.contentView     = view.view
    }

    func triggerAllBoundsChanged() {
        // Notify all the FloViews that the bounds have changed
        var remainingViews = [_window.contentView!]

        while let nextView = remainingViews.popLast() {
            // Trigger the bounds changed event on any container views
            if let containerView = nextView as? FloContainerView {
                containerView.triggerBoundsChanged()
            }

            // Process the entire view tree
            remainingViews.append(contentsOf: nextView.subviews)
        }
    }

    ///
    /// The backing properties (colour scheme, resolution) of the window was changed
    ///
    @objc public func windowDidChangeBackingProperties(_ notification: Notification) {
        triggerAllBoundsChanged()
    }

    ///
    /// Callback when a tick has occurred
    ///
    @objc func tick() {
        if let session = _session {
            // The Rust obj-c crate doesn't provide a way to generate the linker symbols necessary to call FloControl directly
            session.perform(#selector(tick))
        }
    }

    ///
    /// Request for a tick event to be generated
    ///
    @objc public func requestTick() {
        // Cocoa doesn't really have a way to request an animation frame other than by delaying. We'll use a delay indicating 120fps here
        RunLoop.main.perform(inModes: [.default, .eventTracking, .modalPanel],
            block: {
                    self.perform(#selector(self.tick),
                                 with: nil,
                                 afterDelay: TimeInterval(1.0 / 120.0),
                                 inModes: [.default, .eventTracking, .modalPanel])
                }
            )
    }
}
