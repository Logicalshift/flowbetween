//
//  FloPopupWindowBackingVIew.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 13/02/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

///
/// The backing view for a popup window
///
class FloPopupWindowBackingView : NSView {
    fileprivate let _borderWidth = CGFloat(8.0);
    fileprivate let _beakWidth = CGFloat(16.0);
    fileprivate let _beakHeight = CGFloat(8.0);
    
    override func setFrameSize(_ newSize: NSSize) {
        subviews.forEach { subview in
            subview.frame = self.bounds;
        };
    }
    
    /// The popup direction for this view
    var direction: PopupDirection = .Above;
    
    ///
    /// Calulates the required size for this view given a content size and a direction
    ///
    func sizeForContentSize(_ contentSize: NSSize, _ direction: PopupDirection) -> NSSize {
        switch (direction) {
        case .Above:            return NSSize(width: contentSize.width + _borderWidth*2, height: contentSize.height + _borderWidth*2 + _beakHeight);
        case .Below:            return NSSize(width: contentSize.width + _borderWidth*2, height: contentSize.height + _borderWidth*2 + _beakHeight);
        case .Left:             return NSSize(width: contentSize.width + _borderWidth*2 + _beakHeight, height: contentSize.height + _borderWidth*2);
        case .Right:            return NSSize(width: contentSize.width + _borderWidth*2 + _beakHeight, height: contentSize.height + _borderWidth*2);
        case .OnTop:            return NSSize(width: contentSize.width + _borderWidth*2, height: contentSize.height + _borderWidth*2);
        case .WindowCentered:   return NSSize(width: contentSize.width + _borderWidth*2, height: contentSize.height + _borderWidth*2);
        case .WindowTop:        return NSSize(width: contentSize.width + _borderWidth*2, height: contentSize.height + _borderWidth*2);
        }
    }
    
    /// This view is not opaque
    override var isOpaque: Bool {
        get {
            return false;
        }
    }
    
    
}
