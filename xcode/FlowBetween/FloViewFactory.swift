//
//  FloViewFactory.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 06/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

@objc public class FloViewFactory : NSObject {
    ///
    /// Creates an empty view
    ///
    @objc public static func createAsEmpty() -> FloView {
        let view = FloView.init();

        view.layer!.backgroundColor = NSColor.init(deviceRed: 1.0, green: 1.0, blue: 1.0, alpha: 0.0).cgColor;

        return view;
    }
    
    ///
    /// Creates an empty view
    ///
    @objc public static func createAsButton() -> FloView {
        let button = NSButton.init(title: "", target: nil, action: #selector(FloView.onClick));
        
        let view = FloView.init(withControl: button);
        button.target = view;
        
        return view;
    }
}
