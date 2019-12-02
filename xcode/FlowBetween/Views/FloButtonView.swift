//
//  FloButtonView.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 03/02/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Foundation

class FloButtonView : FloControlView {
    /// Sets the text label for this view
    override func setTextLabel(label: String) {
        if let button = _control as? NSButton {
            // NSButtons have the stringValue property but also have a title property where the text
            // is actually set
            button.title = label;
        } else {
            _control.stringValue = label;
        }
    }

    /// Sets part of the state of this control
    override func setState(selector: ViewStateSelector, toProperty: FloProperty) {
        viewState.retainProperty(selector: selector, property: toProperty);

        switch (selector) {
        case .Value:
            weak var this = self;
            toProperty.trackValue { value in
                switch (value) {
                case .Bool(let isSelected):
                    let state: NSButton.StateValue;
                    if isSelected {
                        state = NSButton.StateValue.on;
                    } else {
                        state = NSButton.StateValue.off;
                    }

                    (this?._control as? NSButton)?.state = state;
                    break;

                case .Float(let floatVal):      this?._control.doubleValue = Double(floatVal); break;
                case .Int(let intValue):        this?._control.intValue = Int32(intValue); break;
                case .String(let stringValue):  this?._control.stringValue = stringValue; break;
                default:                        break;
                }
            }
            break;

        case .Selected:
            weak var this = self;
            toProperty.trackValue { value in
                switch (value) {
                case .Bool(let isSelected):
                    let state: NSButton.StateValue;
                    if isSelected {
                        state = NSButton.StateValue.on;
                    } else {
                        state = NSButton.StateValue.off;
                    }

                    (this?._control as? NSButton)?.state = state;
                    break;

                    default:                        break;
                }
            }
            break;

        default:
            super.setState(selector: selector, toProperty: toProperty);
        }
    }
}
