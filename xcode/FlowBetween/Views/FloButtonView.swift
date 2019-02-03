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
}
