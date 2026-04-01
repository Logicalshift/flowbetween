//
//  FloContainerPopup.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 13/02/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Foundation

///
/// The direction a popup window is displayed in relative to its parent view
///
enum PopupDirection : UInt32 {
    case OnTop           = 0
    case Left            = 1
    case Right           = 2
    case Above           = 3
    case Below           = 4
    case WindowCentered  = 5
    case WindowTop       = 6
}

///
/// Extra methods for the container view implemented by popup views
///
protocol FloContainerPopup : FloContainerView {
    /// Sets whether or not the popup view is open
    func setPopupOpen(_ isOpen: Bool)

    /// Sets the direction that the popup window appears in relative to the parent window
    func setPopupDirection(_ direction: PopupDirection)

    /// Sets the sisze of the popup
    func setPopupSize(width: CGFloat, height: CGFloat)

    /// Sets the offset of the popup in the popup direction
    func setPopupOffset(_ offset: CGFloat)
}
