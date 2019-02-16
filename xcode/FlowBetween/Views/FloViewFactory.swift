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
        
        let view        = FloView.init(withView: buttonView);
        button.target   = view;
        
        return view;
    }
    
    ///
    /// Creates a view that acts like a button but can contain other views
    ///
    @objc public static func createAsContainerButton() -> FloView {
        let buttonView  = FloContainerButton(frame: NSRect());
        let view        = FloView.init(withView: buttonView);
        
        return view;
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
        let slider      = NSSlider.init(value: 0.5, minValue: 0.0, maxValue: 1.0, target: nil, action: nil);
        let sliderView  = FloControlView.init(frame: CGRect.init(), control: slider);
        
        let view        = FloView.init(withView: sliderView);
        
        return view;
    }
    
    ///
    /// Creates a view that can be checked on or off
    ///
    @objc public static func createAsCheckBox() -> FloView {
        // Create a checkbox
        let checkbox        = NSButton.init();
        
        checkbox.setButtonType(NSButton.ButtonType.switch);
        
        // Generate the view
        let checkboxView    = FloButtonView.init(frame: CGRect.init(), control: checkbox);
        let view            = FloView.init(withView: checkboxView);
        
        return view;
    }
    
    ///
    /// Creates a view that can contain some text to be edited
    ///
    @objc public static func createAsTextBox() -> FloView {
        let textbox         = NSTextField.init();
        textbox.font        = NSFontManager.shared.font(withFamily: "Lato", traits: NSFontTraitMask(), weight: 5, size: 13.0);
        textbox.isEditable  = true;
        textbox.isBordered  = false;
        textbox.isBezeled   = false;
        let textboxView     = FloControlView.init(frame: CGRect.init(), control: textbox);

        let view            = FloView.init(withView: textboxView);

        return view;
    }
    
    ///
    /// Creates a view that shows a pop-up window
    ///
    @objc public static func createAsPopup() -> FloView {
        let popup   = FloPopupView.init(frame: CGRect.init());
        let view    = FloView.init(withView: popup);
        
        return view;
    }
}
