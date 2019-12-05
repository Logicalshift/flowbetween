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
        let view = FloView()

        view.view.layer!.backgroundColor = NSColor(deviceRed: 1.0, green: 1.0, blue: 1.0, alpha: 0.0).cgColor

        return view
    }

    ///
    /// Creates a view with a button
    ///
    @objc public static func createAsButton() -> FloView {
        let button      = NSButton(title: "", target: nil, action: #selector(FloView.onClick))
        let buttonView  = FloButtonView(frame: .zero, control: button)

        let view        = FloView(withView: buttonView)
        button.target   = view
        button.action   = #selector(FloView.onClick)

        return view
    }

    ///
    /// Creates a view that acts like a button but can contain other views
    ///
    @objc public static func createAsContainerButton() -> FloView {
        let buttonView  = FloContainerButton(frame: .zero)
        let view        = FloView(withView: buttonView)

        return view
    }

    ///
    /// Creates a view that can be scrolled
    ///
    @objc public static func createAsScrolling() -> FloView {
        let scrolling = FloScrollingView()

        let view = FloView(withView: scrolling)

        return view
    }

    ///
    /// Creates a view that can adjust a value
    ///
    @objc public static func createAsSlider() -> FloView {
        let slider      = NSSlider(value: 0.5, minValue: 0.0, maxValue: 1.0, target: nil, action: nil)
        let sliderView  = FloControlView(frame: .zero, control: slider)

        let view        = FloView(withView: sliderView)

        return view
    }

    ///
    /// Creates a view that works like a slider but rotates its contents
    ///
    @objc public static func createAsRotor() -> FloView {
        let rotor   = FloRotorView(frame: .zero)
        let view    = FloView(withView: rotor)

        return view
    }

    ///
    /// Creates a view that can be checked on or off
    ///
    @objc public static func createAsCheckBox() -> FloView {
        // Create a checkbox
        let checkbox        = NSButton()

        checkbox.setButtonType(NSButton.ButtonType.switch)

        // Generate the view
        let checkboxView    = FloButtonView(frame: .zero, control: checkbox)
        let view            = FloView(withView: checkboxView)

        return view
    }

    ///
    /// Creates a view that can contain some text to be edited
    ///
    @objc public static func createAsTextBox() -> FloView {
        let textbox         = NSTextField()
        textbox.font        = NSFontManager.shared.font(withFamily: "Lato", traits: NSFontTraitMask(), weight: 5, size: 13.0)
        textbox.isEditable  = true
        textbox.isBordered  = false
        textbox.isBezeled   = false

        let textboxView     = FloControlView(frame: .zero, control: textbox)
        let view            = FloView(withView: textboxView)

        textbox.delegate    = textboxView

        return view
    }

    ///
    /// Creates a view that shows a pop-up window
    ///
    @objc public static func createAsPopup() -> FloView {
        let popup   = FloPopupView(frame: .zero)
        let view    = FloView(withView: popup)

        return view
    }
}
