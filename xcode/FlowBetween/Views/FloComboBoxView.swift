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
            // Clear the combo box (first item is the title item)
            if comboBox.numberOfItems > 0 {
                // First item is secretly the title of the pulldown (pulldowns are comboboxes underneath so have a bunch of weird behaviour)
                for idx in 1..<comboBox.numberOfItems {
                    comboBox.removeItem(at: idx)
                }
            }
            
            // Before we add the elements, the title must not be set to a valid name
            // Pulldowns allow the title to be the same as an element name BUT comboboxes
            // do not allow two items with the same name. When we add a new element, the
            // combobox behaviour occurs if the title is set to the name of an element.
            // This removes the title element to replace it with the new element, which
            // effectively just changes the title to the first choice...
            // (This only occurs if the title matches an element in the list)
            comboBox.setTitle("")
            
            // Add the choices in turn
            choices.forEach({ choice in comboBox.addItem(withTitle: choice) })
            
            // Set the title back again (avoiding the dumb behaviour described above)
            comboBox.setTitle(self.title)
        }
    }
    
    /// User activated the control (selected an item from the menu)
    override func controlAction() {
        // User clicked one of the menu options
        if let comboBox = _control as? NSPopUpButton {
            let selectedIndex = comboBox.indexOfSelectedItem
            
            // Index 0 is the title item
            if selectedIndex > 0 {
                if let onClick = onClickOption[uint32(selectedIndex-1)] {
                    let _ = onClick()
                }
            }
        }
    }
}
