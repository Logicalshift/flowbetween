//
//  FloComboBoxView.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 23/12/2021.
//  Copyright © 2021 Andrew Hunter. All rights reserved.
//

import Foundation

class FloComboBoxView : FloControlView {
    /// Sets the text label for this view
    override func setTextLabel(label: String) {
        if let comboBox = _control as? NSPopUpButton {
            comboBox.addItem(withTitle: "Test")
            comboBox.setTitle(label)
        } else {
            _control.stringValue = label
        }
    }
}
