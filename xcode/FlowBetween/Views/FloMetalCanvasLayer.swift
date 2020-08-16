//
//  FloMetalCanvasLayer.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 14/08/2020.
//  Copyright © 2020 Andrew Hunter. All rights reserved.
//

import Cocoa
import Metal

///
/// Layer that renders a canvas using GPU accelleration from Metal
///
class FloMetalCanvasLayer : CAMetalLayer {
    /// False until a visible area has been set
    fileprivate var _ready: Bool
    
    /// True if a draw action is pending
    fileprivate var _drawPending: Bool

    /// The events for this layer
    fileprivate weak var _events: FloEvents?
    
    /// The visibile area of the canvas
    fileprivate var _visibleRect: CGRect
    
    /// The total size of the canvas
    fileprivate var _size: CGSize
    
    /// The resolution multiplier
    fileprivate var _resolution: CGFloat
    
    init(events: FloEvents) {
        _ready          = false
        _drawPending    = false
        _events         = events
        _visibleRect    = CGRect(x: 0, y: 0, width: 500, height: 500)
        _size           = CGSize(width: 500, height: 500)
        _resolution     = 1.0

        super.init()
    }
    
    override init(layer: Any) {
        _ready          = false
        _drawPending    = false
        _events         = nil
        _visibleRect    = CGRect(x: 0, y: 0, width: 1, height: 1)
        _size           = CGSize(width: 1, height: 1)
        _resolution     = 1.0

        super.init(layer: layer);

        if let layer = layer as? FloMetalCanvasLayer {
            _ready          = layer._ready
            _events         = layer._events
            _visibleRect    = layer._visibleRect
            _size           = layer._size
            _resolution     = layer._resolution
        }
    }

    required init?(coder aDecoder: NSCoder) {
        _ready          = false
        _drawPending    = false
        _events         = nil
        _visibleRect    = CGRect(x: 0, y: 0, width: 1, height: 1)
        _size           = CGSize(width: 1, height: 1)
        _resolution     = 1.0

        super.init(coder: aDecoder)
    }
    
    ///
    /// Queues a redraw event if none is pending presently
    ///
    func queueRedraw() {
        // Queue a redraw if none is pending
        if !_drawPending {
            _drawPending = true;
            RunLoop.main.perform(inModes: [.default, .modalPanel, .eventTracking], block: {
                if self._drawPending {
                    self._drawPending = false
                    self.performRedraw()
                }
            })
        }
    }
    
    ///
    /// Performs a redraw immediately
    ///
    func performRedraw() {
        // Cancel any other redraws that might be queued
        _drawPending = false;
        
        // Do not draw anything if the view's visible area has not been set
        if !_ready {
            return
        }
        
        // Fetch the next drawable
        if let flo_events = _events {
            let drawable = nextDrawable()
            
            if var drawable = drawable {
                let unsafe_drawable = AutoreleasingUnsafeMutablePointer<CAMetalDrawable?>(&drawable)
                
                // Send it to be redrawn via the events
                flo_events.redrawGpuCanvas(with: unsafe_drawable,
                       size: _size,
                       viewport: _visibleRect,
                       resolution: _resolution);
            }
        }
    }

    ///
    /// Updates the area of the canvas that this layer should display
    ///
    func setVisibleArea(bounds: ContainerBounds, resolution: CGFloat) {
        autoreleasepool {
            _ready                  = true
            _size                   = bounds.totalSize
            _visibleRect            = bounds.visibleRect
            
            _visibleRect.origin.y   = _size.height - (bounds.visibleRect.maxY)
            
            _resolution             = resolution
            contentsScale           = resolution
        }
    }
}
