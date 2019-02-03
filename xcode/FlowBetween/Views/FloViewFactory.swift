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

        view.view.layer!.backgroundColor = NSColor.init(deviceRed: 1.0, green: 1.0, blue: 1.0, alpha: 0.0).cgColor;

        return view;
    }
    
    ///
    /// Creates a view with a button
    ///
    @objc public static func createAsButton() -> FloView {
        let button      = NSButton.init(title: "", target: nil, action: #selector(FloView.onClick));
        let buttonView  = FloButtonView.init(frame: CGRect.init(), control: button);
        
        let view = FloView.init(withView: buttonView);
        button.target = view;
        
        return view;
    }
    
    ///
    /// Creates a view that acts like a button but can contain other views
    ///
    @objc public static func createAsContainerButton() -> FloView {
        return createAsEmpty();
    }
    
    ///
    /// Creates a view that can be scrolled
    ///
    @objc public static func createAsScrolling() -> FloView {
        let scrolling = FloScrollingView.init();
        
        let view = FloView.init(withView: scrolling);
        
        return view;
    }
    
    ///
    /// Creates a view that can adjust a value
    ///
    @objc public static func createAsSlider() -> FloView {
        return createAsEmpty();
    }
    
    ///
    /// Creates a view that can be checked on or off
    ///
    @objc public static func createAsCheckBox() -> FloView {
        return createAsEmpty();
    }
    
    ///
    /// Creates a view that can contain some text to be edited
    ///
    @objc public static func createAsTextBox() -> FloView {
        return createAsEmpty();
    }
}
