//
//  AppDelegate.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 02/01/2019.
//  Copyright © 2019 Andrew Hunter.
//
// FlowBetween is distributed under the terms of the Apache public license
//
//    Copyright 2018-2019 Andrew Hunter
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

import Cocoa

@NSApplicationMain
class FloAppDelegate: NSObject, NSApplicationDelegate {
    /// The FloSession object
    var _sessions: [UInt64: NSObject] = [UInt64: NSObject]();
    
    /// Views requesting 'dismiss' events
    var _dismiss: [FloViewWeakRef] = [];

    func applicationDidFinishLaunching(_ aNotification: Notification) {
        weak var this = self;
        
        // Monitor events to generate the 'dismiss' action
        NSEvent.addLocalMonitorForEvents(matching: NSEvent.EventTypeMask.leftMouseDown
            .union(.otherMouseDown)
            .union(.rightMouseDown)
            .union(.tabletProximity)
            .union(.tabletPoint),
                                         handler: { event in this?.monitorEvent(event); return event; })
        
        // Create the Flo session
        let session             = create_flo_session(FloWindowDelegate.self, FloViewFactory.self, FloViewModel.self);

        let sessionId           = session!.sessionId();
        _sessions[sessionId]    = session;
    }

    func applicationWillTerminate(_ aNotification: Notification) {
    }
    
    ///
    /// Requests that dismiss events are sent to the specified view
    ///
    func requestDismiss(forView: FloView) {
        // Clear up any views that are no longer in the list
        _dismiss.removeAll(where: { view in view.floView == nil });
        
        // Add the view to the list that should have dismiss requests sent
        _dismiss.append(FloViewWeakRef(floView: forView));
    }
    
    ///
    /// Sends a dismiss event to any view outside of the specified view's hierarchy
    ///
    func sendDismiss(forView: NSView?) {
        // List of FloViews to dismiss
        _dismiss.removeAll(where: { view in view.floView == nil });
        var toDismiss = _dismiss;
        
        // Nothing to do if there are no dismissable views
        if toDismiss.count <= 0 {
            return;
        }
        
        // Iterate through the view hierarchy, and remove view
        var window      = forView?.window;
        var superview   = forView;
        while let view = superview {
            // If the click is inside a 'dismissable' view, then don't dismiss that view
            if let containerView = view as? FloContainerView {
                toDismiss.removeAll(where: { view in view.floView == containerView.floView });
            }
            
            // Move up the hierarchy
            superview = view.superview;
            
            if superview == nil {
                if let popupWindow = window as? FloPopupWindow {
                    superview   = popupWindow.parentView;
                    window      = superview?.window;
                }
            }
        }
        
        // Request all remaining dismissable views dismiss themselves
        toDismiss.forEach({ view in view.floView?.sendDismiss() });
    }
    
    ///
    /// Monitors an event sent to the application
    ///
    func monitorEvent(_ event: NSEvent) {
        switch event.type {
        case .tabletPoint:
            NSLog("tabletPoint: \(event.pointingDeviceType.rawValue)");
            NSLog("tabletPoint: \(event.pointingDeviceID)");
            break;
            
        case .tabletProximity:
            NSLog("tabletProx: \(event.pointingDeviceType.rawValue)");
            NSLog("tabletProx: \(event.pointingDeviceID)");
            break;
            
        case .leftMouseDown, .otherMouseDown, .rightMouseDown:
            if event.subtype == .tabletPoint || event.subtype == .tabletProximity {
                NSLog("tabletMouse: \(event.pointingDeviceType.rawValue)");
                NSLog("tabletMouse: \(event.pointingDeviceID)");
            }
            break;
            
        default: break;
        }
        
        if _dismiss.count == 0 {
            // Short-circuit the check if there are no dismissable views
            return;
        }
        
        switch event.type {
        case .leftMouseDown, .otherMouseDown, .rightMouseDown:
            // Mouse down in the window
            if let window = event.window {
                // Find out the view that the user clicked on
                let locationInWindow    = event.locationInWindow;
                let hitView             = window.contentView?.hitTest(locationInWindow);
                
                // Send the dismiss event
                sendDismiss(forView: hitView);
            } else {
                // Mouse down in no view
                sendDismiss(forView: nil);
            }
            break;
        
        default:
            // Do nothing
            break;
        }
    }
    
    ///
    /// User requested a new session
    ///
    @IBAction public func newDocument(_ sender: Any?) {
        // Create the Flo session
        let session             = create_flo_session(FloWindowDelegate.self, FloViewFactory.self, FloViewModel.self);

        let sessionId           = session!.sessionId();
        _sessions[sessionId]    = session;
    }
    
    ///
    /// A particular session has finished
    ///
    func finishSessionWithId(_ sessionId: UInt64) {
        _sessions.removeValue(forKey: sessionId);
    }
}

