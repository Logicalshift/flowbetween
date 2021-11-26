//
//  FloImageView.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 21/10/2020.
//  Copyright © 2020 Andrew Hunter. All rights reserved.
//

import Cocoa

///
/// View that displays an image set on a control
///
class FloImageView : NSView {
    /// The image that this will display
    fileprivate var _image: NSImage?;

    /// Stores the general state of this view
    var viewState : ViewState = ViewState()

    required override init(frame frameRect: NSRect) {
        super.init(frame: frameRect)

        wantsLayer              = true
        layer!.contentsGravity  = CALayerContentsGravity.resizeAspect
    }

    required init?(coder decoder: NSCoder) {
        super.init(coder: decoder)

        wantsLayer              = true
        layer!.contentsGravity  = CALayerContentsGravity.resizeAspect
    }

    ///
    /// The hit testing routine is used to make the view 'click-through' if required
    ///
    override func hitTest(_ point: NSPoint) -> NSView? {
        // Check what view has been clicked on
        let hitTestResult = super.hitTest(point);
        
        if hitTestResult == self {
            // If the click is directly on this view, then return this view only if it's not a 'click through' view
            if viewState.clickThrough {
                return nil;
            } else {
                return hitTestResult;
            }
        } else {
            // If another view has claimed the click, then don't suppress it
            return hitTestResult;
        }
    }

    ///
    /// Updates the image that this view is displaying
    ///
    fileprivate func updateImage() {
        if let screen = window?.screen {
            // Update the layer
            let resolution          = screen.backingScaleFactor
            layer!.contentsScale    = resolution
            
            // Set the contents to the current image
            layer!.contents         = _image?.layerContents(forContentsScale: resolution)
        } else {
            // Not on screen
            let resolution          = CGFloat(1.0)
            layer!.contentsScale    = resolution
            
            // Set the contents to the current image
            layer!.contents         = _image?.layerContents(forContentsScale: resolution)
        }
    }
    
    override func setFrameSize(_ newSize: NSSize) {
        super.setFrameSize(newSize)
        self.updateImage()
    }
    
    var image : NSImage? {
        get { return _image }
        set(value) {
            _image = value
            updateImage()
        }
    }
}
