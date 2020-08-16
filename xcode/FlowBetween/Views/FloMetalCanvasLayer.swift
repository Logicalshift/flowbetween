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
    /// True if a draw action is pending
    fileprivate var _drawPending: Bool;

    /// The events for this layer
    fileprivate weak var _events: FloEvents?;
    
    init(events: FloEvents) {
        _drawPending    = false;
        _events         = events;

        super.init()
    }
    
    override init(layer: Any) {
        _drawPending    = false;
        _events         = nil;
        
        super.init(layer: layer);

        if let layer = layer as? FloMetalCanvasLayer {
            _events     = layer._events;
        }
    }

    required init?(coder aDecoder: NSCoder) {
        _drawPending    = false;
        _events         = nil;
        
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
                    self._drawPending = false;
                    self.performRedraw();
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
        
        // Fetch the next drawable
        if let flo_events = _events {
            let drawable = nextDrawable();
            
            if var drawable = drawable {
                let unsafe_drawable = AutoreleasingUnsafeMutablePointer<CAMetalDrawable?>(&drawable);
                
                // Send it to be redrawn via the events
                flo_events.redrawGpuCanvas(with: unsafe_drawable,
                       size: NSSize(width: 1980, height: 1080),
                       viewport: NSRect(x: 0, y: 0, width: 1920, height: 1080));
            }
            
            // Present the drawable after rendering
            drawable?.present();
        }
    }
}
