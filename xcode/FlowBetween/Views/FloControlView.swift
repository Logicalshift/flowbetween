//
//  FloControlView.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 03/02/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

///
/// Container view containing a Cocoa control
///
class FloControlView: NSView, FloContainerView {
    /// The control that is displayed in this view
    let _control: NSControl;

    required init(frame frameRect: NSRect, control: NSControl) {
        _control = control;
        
        super.init(frame: frameRect);
        
        wantsLayer = true;
        
        _control.frame = bounds;
        addSubview(_control);
    }
    
    required init?(coder decoder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
    
    /// Updates the frame size of this view
    override func setFrameSize(_ newSize: NSSize) {
        NSLog("Control frame set to \(newSize)");
        
        super.setFrameSize(newSize);
        _control.frame = bounds;
    }
    
    /// Adds a subview to this container view
    func addContainerSubview(_ subview: NSView) {
        // Control views cannot have subviews (not supported in Cocoa's model)
    }
    
    /// Sets the layer displayed for the canvas
    func setCanvasLayer(_ layer: CALayer) {
        // Control views cannot have layers
    }
    
    /// The size of the layout area for this view
    var layoutSize : NSSize {
        get {
            return self.bounds.size;
        }
    };
    
    /// Returns this view as an NSView
    var asView : NSView { get { return self; } };
    
    /// Event handler: user clicked in the view
    var onClick: (() -> Bool)?;
    
    /// Event handler: user scrolled/resized so that a particular region is visible
    var onScroll: ((NSRect) -> ())?;
    
    /// Events handlers when a particular device is used for painting
    var onPaint: [FloPaintDevice: (FloPaintStage, AppPainting) -> ()] = [FloPaintDevice: (FloPaintStage, AppPainting) -> ()]();
    
    /// The affine transform for the canvas layer
    var canvasAffineTransform: CGAffineTransform?;
    
    /// Event handler: user performed layout on this view
    var performLayout: ((NSSize) -> ())?;
    
    /// Event handler: The bounds of the container have changed
    var boundsChanged: ((ContainerBounds) -> ())?;
    
    /// The minimum size of the scroll area for this view
    var scrollMinimumSize: (Float64, Float64) = (0,0);
    
    /// The visibility of the horizontal and vertical scroll bars
    var scrollBarVisibility: (ScrollBarVisibility, ScrollBarVisibility) = (ScrollBarVisibility.Never, ScrollBarVisibility.Never);
    
    /// Triggers the click event for this view
    func triggerClick() {
        let _ = onClick?()
    }
    
    ///
    /// Computes the container bounds for this view
    ///
    func getContainerBounds() -> ContainerBounds {
        // Get the bounds
        let viewport        = bounds;
        var visible         = visibleRect;
        
        // For the container bounds, the viewport is considered to be aligned at 0,0
        visible.origin.x    -= viewport.origin.x;
        visible.origin.y    -= viewport.origin.y;
        
        return ContainerBounds(visibleRect: visible, totalSize: viewport.size);
    }

    /// Triggers the bounds changed event for this view
    func triggerBoundsChanged() {
        boundsChanged?(getContainerBounds());
    }
    
    /// Sets the text label for this view
    func setTextLabel(label: String) {
        _control.stringValue = label;
    }
    
    /// Sets the font size for this view
    func setFontSize(points: Float64) {
        let existingFont    = _control.font!;
        let newFont         = NSFontManager.shared.convert(existingFont, toSize: CGFloat(points));
        
        _control.font       = newFont;
    }

    ///
    /// Converts a weight from a value like 100, 200, 400, etc to a font manager weight (0-15)
    ///
    func convertWeight(_ weight: Float64) -> Int {
        if weight <= 150.0 {
            return 1;
        } else if weight <= 450.0 {
            return 5;
        } else if weight <= 750.0 {
            return 7;
        } else {
            return 10;
        }
    }

    /// Sets the font weight for this view
    func setFontWeight(weight: Float64) {
        let existingFont        = _control.font!;
        let fontManagerWeight   = convertWeight(weight);
        let family              = existingFont.familyName!;
        let size                = existingFont.pointSize;
        let traits              = NSFontTraitMask();
        
        let newFont             = NSFontManager.shared.font(withFamily: family, traits: traits, weight: fontManagerWeight, size: size);
        
        _control.font           = newFont;
    }
    
    /// Sets the text alignment for this view
    func setTextAlignment(alignment: NSTextAlignment) {
        _control.alignment = alignment;
    }
}
