//
//  FloCanvasLayer.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 25/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

///
/// Layer that renders a canvas
///
class FloCanvasLayer : CALayer {
    /// The backing for this layer (nil if it's not drawable yet)
    var _backing: CGLayer?;
    
    /// Function called to trigger a redraw
    var _triggerRedraw: ((NSSize, NSRect) -> ())?;
    
    /// The overall size of the canvas
    var _canvasSize: NSSize;
    
    /// The coordinates of the visible region in the canvsa
    var _visibleRect: NSRect;
    
    /// The resolution of this layer
    var _resolution: CGFloat = 1.0;
    
    override init() {
        _canvasSize     = NSSize(width: 1, height: 1);
        _visibleRect    = NSRect(x: 0, y: 0, width: 1, height: 1);
        
        super.init();
    }
    
    override init(layer: Any) {
        _canvasSize     = NSSize(width: 1, height: 1);
        _visibleRect    = NSRect(x: 0, y: 0, width: 1, height: 1);

        super.init();
        
        if let layer = layer as? FloCanvasLayer {
            _backing            = layer._backing;
            _canvasSize         = layer._canvasSize;
            _visibleRect        = layer._visibleRect;
            _resolution         = layer._resolution;
        }
    }
    
    required init?(coder aDecoder: NSCoder) {
        _canvasSize     = NSSize(width: 1, height: 1);
        _visibleRect    = NSRect(x: 0, y: 0, width: 1, height: 1);

        super.init(coder: aDecoder);
    }
    
    override func draw(in ctx: CGContext) {
        // Redraw the backing layer if it has been invalidated
        if _backing == nil {
            var size    = _visibleRect.size;
            size.width  *= _resolution;
            size.height *= _resolution;
            
            if size.width == 0 { size.width = 1; }
            if size.height == 0 { size.height = 1; }
            
            // Create the backing layer
            _backing = CGLayer(ctx, size: size, auxiliaryInfo: nil);
            
            if _resolution != 1.0 {
                let scale = CGAffineTransform.init(scaleX: _resolution, y: _resolution);
                _backing!.context!.concatenate(scale);
            }
            
            // Force a redraw via the events
            _triggerRedraw?(_canvasSize, _visibleRect);
        }
        
        // Draw the backing layer on this layer
        if let backing = _backing {
            ctx.draw(backing, in: self.bounds);
        }
    }
    
    ///
    /// Clears the backing layers for this layer
    ///
    func clearBackingLayers() {
        _backing?.context?.clear(CGRect(origin: CGPoint(x: 0, y: 0), size: self.bounds.size));
    }
}
