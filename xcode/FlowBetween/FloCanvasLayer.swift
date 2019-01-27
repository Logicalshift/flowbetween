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
    var _backing: [UInt32: CGContext];
    
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
        _backing        = [UInt32: CGContext]();

        super.init();
    }
    
    override init(layer: Any) {
        _canvasSize     = NSSize(width: 1, height: 1);
        _visibleRect    = NSRect(x: 0, y: 0, width: 1, height: 1);
        _backing        = [UInt32: CGContext]();

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
        _backing        = [UInt32: CGContext]();

        super.init(coder: aDecoder);
    }
    
    override func draw(in ctx: CGContext) {
        autoreleasepool {
            // Draw the backing layer on this layer
            let layer_ids   = _backing.keys.sorted();
            let bounds      = self.bounds;
            
            for layer_id in layer_ids {
                if let image = _backing[layer_id]!.makeImage() {
                    ctx.draw(image, in: bounds);
                }
            }
        }
    }
    
    ///
    /// Clears the backing layers for this layer
    ///
    func clearBackingLayers() {
        // All layers other than layer 0 are removed
        let layers_to_remove = _backing.keys.filter({ layer_id in layer_id != 0 });
        for layer_id in layers_to_remove {
            _backing.removeValue(forKey: layer_id);
        }
        
        // Clear the bottom layer
        _backing[0]?.clear(CGRect(origin: CGPoint(x: 0, y: 0), size: self.bounds.size));
    }
    
    ///
    /// Ensures the layer with the specifed ID exists
    ///
    func ensureLayerWithId(id: UInt32) {
        if !_backing.keys.contains(id) {
            // Get the size for the new layer
            var size    = _visibleRect.size;
            size.width  *= _resolution;
            size.height *= _resolution;
            
            if size.width == 0 { size.width = 1; }
            if size.height == 0 { size.height = 1; }

            // We create the new layer from a base layer (as CGLayer needs a context to work from)
            let newLayer = CGContext.init(data:             nil,
                                          width:            Int(size.width),
                                          height:           Int(size.height),
                                          bitsPerComponent: 8,
                                          bytesPerRow:      0,
                                          space:            CGColorSpaceCreateDeviceRGB(),
                                          bitmapInfo:       CGImageAlphaInfo.premultipliedLast.rawValue);
            
            if _resolution != 1.0 {
                let scale = CGAffineTransform.init(scaleX: _resolution, y: _resolution);
                newLayer!.concatenate(scale);
            }
            
            newLayer?.setFillColor(red: 1, green: 0, blue: 0, alpha: 1);
            newLayer?.fill(self.bounds);
            
            // Store the new layer as a new backing layer
            _backing[id] = newLayer!;
        }
    }
}
