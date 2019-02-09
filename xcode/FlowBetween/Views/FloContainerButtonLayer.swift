//
//  FloContainerButtonLayer.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 09/02/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

///
/// A layer representing the background of a conainer button
///
class FloContainerButtonLayer : CALayer {
    fileprivate var _highlighted    = false;
    fileprivate var _selected       = false;
    
    /// Set to true if this button is highlighted
    var highlighted: Bool {
        get { return _highlighted; }
        set(value) { _highlighted = value; setNeedsDisplay(); }
    }
    
    /// Set to true if this button is selected
    var selected: Bool {
        get { return _selected; }
        set(value) { _selected = value; setNeedsDisplay(); }
    }
    
    override func resize(withOldSuperlayerSize size: CGSize) {
        setNeedsDisplay();
        super.resize(withOldSuperlayerSize: size);
    }
    
    /// Draws the content of this layer
    override func draw(in ctxt: CGContext) {
        let background: CGColor;
        let border:     CGColor;
        
        // Colours are based on whether or not we're highlighted or selected
        if highlighted && selected {
            border      = CGColor.init(red: 0.5, green: 0.6, blue: 0.7, alpha: 1.0);
            background  = CGColor.init(red: 0.0, green: 0.7, blue: 0.9, alpha: 1.0);
        } else if selected {
            border      = CGColor.init(red: 0.5, green: 0.6, blue: 0.7, alpha: 1.0);
            background  = CGColor.init(red: 0.0, green: 0.2, blue: 0.5, alpha: 0.8);
        } else if highlighted {
            border      = CGColor.clear;
            background  = CGColor.init(red: 0.7, green: 0.7, blue: 0.8, alpha: 0.5);
        } else {
            border      = CGColor.clear;
            background  = CGColor.init(red: 0.4, green: 0.4, blue: 0.4, alpha: 0.2);
        }
        
        // Draw the button background
        let rounded = CGPath.init(roundedRect: bounds.insetBy(dx: 2.0, dy: 2.0), cornerWidth: 6.0, cornerHeight: 6.0, transform: nil);
        ctxt.beginPath();
        ctxt.addPath(rounded);
        ctxt.setFillColor(background);
        ctxt.fillPath();

        // And the border
        ctxt.beginPath();
        ctxt.addPath(rounded);
        ctxt.setStrokeColor(border);
        ctxt.setLineWidth(1.5);
        ctxt.strokePath();
    }
}
