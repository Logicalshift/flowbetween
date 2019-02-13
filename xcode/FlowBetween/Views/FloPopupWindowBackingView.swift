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
        super.setFrameSize(newSize);
        
        // Subview bounds depend on the direction
        var subviewBounds = CGRect(origin: CGPoint(), size: newSize).insetBy(dx: _borderWidth, dy: _borderWidth);
        
        if newSize.width < _borderWidth*2 || newSize.height < _borderWidth*2 {
            subviewBounds = CGRect();
        }
        
        switch (direction) {
        case .Above:    subviewBounds.origin.y += _beakHeight; subviewBounds.size.height -= _beakHeight; break;
        case .Below:    subviewBounds.size.height -= _beakHeight; break;
        case .Left:     subviewBounds.origin.x += _beakHeight; subviewBounds.size.width -= _beakHeight; break;
        case .Right:    subviewBounds.size.width -= _beakHeight; break;
            
        case .OnTop, .WindowCentered, .WindowTop: break;
        }
        
        // Update the bounds of all the subviews
        subviews.forEach { subview in
            subview.frame = subviewBounds;
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
    
    override func draw(_ dirtyRect: NSRect) {
        let ctxt = NSGraphicsContext.current!.cgContext;
        
        let path = CGPath.init(roundedRect: self.bounds.insetBy(dx: 2.0, dy: 2.0), cornerWidth: 8.0, cornerHeight: 8.0, transform: nil);
        
        ctxt.setFillColor(CGColor.init(red: 0.25, green: 0.2, blue: 0.2, alpha: 0.9));
        ctxt.addPath(path);
        ctxt.fillPath();

        ctxt.setStrokeColor(CGColor.init(red: 0.1, green: 0.1, blue: 0.1, alpha: 0.8));
        ctxt.setLineWidth(4.0);
        ctxt.addPath(path);
        ctxt.strokePath();

        ctxt.setStrokeColor(CGColor.init(red: 0.9, green: 0.9, blue: 0.9, alpha: 0.8));
        ctxt.setLineWidth(3.25);
        ctxt.addPath(path);
        ctxt.strokePath();
    }
}
