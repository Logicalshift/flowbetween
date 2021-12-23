//
//  FloComboBoxView.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 23/12/2021.
//  Copyright © 2021 Andrew Hunter. All rights reserved.
//

import Foundation

class FloComboBoxView : FloControlView {
    fileprivate var title: String = ""
    
    /// Sets the text label for this view
    override func setTextLabel(label: String) {
        if let comboBox = _control as? NSPopUpButton {
            title = label
            comboBox.setTitle(label)
        } else {
            _control.stringValue = label
        }
    }

    /// Sets the menu choices for this view
    override func setMenuChoices(_ choices: [String]) {
        if let comboBox = _control as? NSPopUpButton {
            // Clear the combo box
            comboBox.removeAllItems()
            
            // Need one choice to represent the title
            comboBox.addItem(withTitle: self.title)
            
            // Add the choices in turn
            choices.forEach({ choice in comboBox.addItem(withTitle: choice) })
        }
    }
}
