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
// Licensed under the Apache License, Version 2.0 (the "License")
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
    var _sessions: [UInt64: NSObject] = [UInt64: NSObject]()

    /// Views requesting 'dismiss' events
    var _dismiss: [FloViewWeakRef] = []

    func applicationDidFinishLaunching(_ aNotification: Notification) {
        weak var this = self

        // Monitor events to generate the 'dismiss' action
        NSEvent.addLocalMonitorForEvents(matching: [.leftMouseDown, .otherMouseDown, .rightMouseDown, .tabletProximity, .tabletPoint],
         handler: { event in this?.monitorEvent(event); return event })

        // Create the Flo session
        let session             = create_flo_session(FloWindowDelegate.self, FloViewFactory.self, FloViewModel.self)

        let sessionId           = session!.sessionId()
        _sessions[sessionId]    = session
    }

    func applicationWillTerminate(_ aNotification: Notification) {
    }

    ///
    /// Requests that dismiss events are sent to the specified view
    ///
    func requestDismiss(forView: FloView) {
        // Clear up any views that are no longer in the list
        _dismiss.removeAll(where: { view in view.floView == nil })

        // Add the view to the list that should have dismiss requests sent
        _dismiss.append(FloViewWeakRef(floView: forView))
    }

    ///
    /// Sends a dismiss event to any view outside of the specified view's hierarchy
    ///
    func sendDismiss(forView: NSView?) {
        // List of FloViews to dismiss
        _dismiss.removeAll(where: { view in view.floView == nil })
        var toDismiss = _dismiss

        // Nothing to do if there are no dismissable views
        if toDismiss.count <= 0 {
            return
        }

        // Iterate through the view hierarchy, and remove view
        var window      = forView?.window
        var superview   = forView
        while let view = superview {
            // If the click is inside a 'dismissable' view, then don't dismiss that view
            if let containerView = view as? FloContainerView {
                toDismiss.removeAll(where: { view in view.floView == containerView.floView })
            }

            // Move up the hierarchy
            superview = view.superview

            if superview == nil {
                if let popupWindow = window as? FloPopupWindow {
                    superview   = popupWindow.parentView
                    window      = superview?.window
                }
            }
        }

        // Request all remaining dismissable views dismiss themselves
        toDismiss.forEach({ view in view.floView?.sendDismiss() })
    }

    ///
    /// The last tablet device to generate a tablet proximity event
    ///
    /// These aren't relayed to views so we capture them at the application level instead.
    /// The tablet mouse event subtypes *are* generated but they don't contain a pointing
    /// device type most of the time, so this can be used to work out which tool the user
    /// is using instead of 'unknown' (basically required to make the eraser work, at least
    /// with Wacom's drivers)
    ///
    var currentTabletPointingDeviceType: NSEvent.PointingDeviceType = .unknown

    ///
    /// The serial number for the current pointing device, for telling the different between
    /// different tablet tools.
    ///
    fileprivate var _currentTabletPointingDeviceSerial: Int = 0

    ///
    /// Monitors an event sent to the application
    ///
    func monitorEvent(_ event: NSEvent) {
        // Track which tablet tool is currently in proximity (Wacom's drivers in particular don't supply this information with every event: see getCurrentTabletPointingDevice for where this is used)
        switch event.type {
        case .tabletProximity:
            if event.isEnteringProximity {
                currentTabletPointingDeviceType    = event.pointingDeviceType
                _currentTabletPointingDeviceSerial  = event.pointingDeviceSerialNumber
            } else {
                currentTabletPointingDeviceType    = .unknown
                _currentTabletPointingDeviceSerial  = 0
            }
            break

        default: break
        }

        if _dismiss.count == 0 {
            // Short-circuit the check if there are no dismissable views
            return
        }

        // Any mouse event outside of something waiting for dismissal should generate a 'dismiss' event
        switch event.type {
        case .leftMouseDown, .otherMouseDown, .rightMouseDown:
            // Mouse down in the window
            if let window = event.window {
                // Find out the view that the user clicked on
                let locationInWindow    = event.locationInWindow
                let hitView             = window.contentView?.hitTest(locationInWindow)

                // Send the dismiss event
                sendDismiss(forView: hitView)
            } else {
                // Mouse down in no view
                sendDismiss(forView: nil)
            }

        default:
            // Do nothing
            break
        }
    }

    ///
    /// User requested a new session
    ///
    @IBAction public func newDocument(_ sender: Any?) {
        // Create the Flo session
        let session             = create_flo_session(FloWindowDelegate.self, FloViewFactory.self, FloViewModel.self)

        let sessionId           = session!.sessionId()
        _sessions[sessionId]    = session
    }

    ///
    /// A particular session has finished
    ///
    func finishSessionWithId(_ sessionId: UInt64) {
        _sessions.removeValue(forKey: sessionId)
    }
}

///
/// Given a pointing device type, either uses that type or if it's not valid, tries to retrieve
/// one from the app delegate.
///
/// Wacom's drivers don't reliably supply this data with tablet actions so we need to track
/// tablet proximity events instead. Cocoa doesn't route this event to views so we capture it
/// in the app delegate instead (see monitorEvent, which also deals with generating the dismiss
/// event)
///
fileprivate func getActivePointingDevice(eventDevice: NSEvent.PointingDeviceType) -> NSEvent.PointingDeviceType {
    if eventDevice == .unknown {
        // Use the last device recorded by the app delegate
        let appDelegate = NSApp.delegate as? FloAppDelegate
        return appDelegate?.currentTabletPointingDeviceType ?? eventDevice
    } else {
        // This device is valid
        return eventDevice
    }
}

///
/// Returns the pointing device being used for a particular NSEvent
///
/// (Wacom's drivers in particular often don't set this field for tablet events, so we track
/// based on proximity instead).
///
func getCurrentTabletPointingDevice(fromEvent: NSEvent) -> NSEvent.PointingDeviceType {
    switch fromEvent.type {
    case .tabletProximity:
        // We generally assume .tabletProximity events have an accurate type
        return fromEvent.pointingDeviceType

    case .tabletPoint:
        // Tablet point events don't always have a valid type (Wacom's drivers again)
        return getActivePointingDevice(eventDevice: fromEvent.pointingDeviceType)

    default:
        switch fromEvent.subtype {
        case .tabletProximity, .tabletPoint:
            // The subtypes may or may not have a valid pointing device type depending on the driver
            // Wacom's drivers only return a valid pointing device type for tabletProximity events.
            return getActivePointingDevice(eventDevice: fromEvent.pointingDeviceType)

        default:
            // Not a tablet event (we claim to be the curver pointing device type here as it's the mouse or something)
            return .cursor
        }
    }
}
