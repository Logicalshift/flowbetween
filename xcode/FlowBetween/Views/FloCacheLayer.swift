//
//  FloCacheLayer.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 02/02/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

///
/// A cache layer manages the stored state of a layer
///
@objc public class FloCacheLayer : NSObject {
    /// The layer managed by this cache
    var _layer: CGLayer

    /// The clear count where it's valid to return this layer to the unused list
    var _clearCount: UInt32

    /// The canvas that this cache layer is for
    weak var _canvas: FloCanvasLayer?

    init(layer: CGLayer, canvas: FloCanvasLayer, clearCount: UInt32) {
        _layer      = layer
        _canvas     = canvas
        _clearCount = clearCount
    }

    deinit {
        // Return this layer to the backing pool of the canvas
        if let canvas = _canvas {
            canvas.returnUnusedLayer(_layer, _clearCount)
        }
    }

    ///
    /// Caches the contents of a layer in this layer
    ///
    func cache(from: CGLayer) {
        if let context = _layer.context {
            // Save the state of the layer
            context.saveGState()

            // Disable scaling, antialiasing, interpolation
            context.setShouldAntialias(false)
            context.interpolationQuality = .none
            context.concatenate(context.ctm.inverted())

            // Copy the layer
            context.setBlendMode(CGBlendMode.copy)
            context.draw(from, at: CGPoint(x: 0, y: 0))

            // Restore to the previous state
            context.restoreGState()
        }
    }

    ///
    /// Restores the contents of this layer to another layer
    ///
    func restore(to: CGLayer) {
        if let context = to.context {
            // Save the state of the layer
            context.saveGState()

            // Disable scaling, antialiasing, interpolation
            context.setShouldAntialias(false)
            context.interpolationQuality = CGInterpolationQuality.none
            context.concatenate(context.ctm.inverted())

            // Copy the layer
            context.setBlendMode(CGBlendMode.copy)
            context.draw(_layer, at: CGPoint(x: 0, y: 0))

            // Restore to the previous state
            context.restoreGState()
        }
    }
}
