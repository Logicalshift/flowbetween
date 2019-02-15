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
    var _floSession: NSObject! = nil;
    
    /// Views requesting 'dismiss' events
    var _dismiss: [FloViewWeakRef] = [];

    func applicationDidFinishLaunching(_ aNotification: Notification) {
        // Insert code here to initialize your application
        _floSession = create_flo_session(FloWindowDelegate.self, FloViewFactory.self, FloViewModel.self);
    }

    func applicationWillTerminate(_ aNotification: Notification) {
        // Insert code here to tear down your application
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
    func sendDismiss(forView: NSView) {
        
    }
}

