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
        _canvas?.returnUnusedLayer(_layer, _clearCount)
    }

    ///
    /// Caches the contents of a layer in this layer
    ///
    func cache(from: CGLayer) {
        guard let context = _layer.context else { return }
        // Save the state of the layer
        context.saveGState()

        // Disable scaling, antialiasing, interpolation
        context.setShouldAntialias(false)
        context.interpolationQuality = .none
        context.concatenate(context.ctm.inverted())

        // Copy the layer
        context.setBlendMode(.copy)
        context.draw(from, at: .zero)

        // Restore to the previous state
        context.restoreGState()
    }

    ///
    /// Restores the contents of this layer to another layer
    ///
    func restore(to: CGLayer) {
        guard let context = to.context else { return }
        // Save the state of the layer
        context.saveGState()

        // Disable scaling, antialiasing, interpolation
        context.setShouldAntialias(false)
        context.interpolationQuality = .none
        context.concatenate(context.ctm.inverted())

        // Copy the layer
        context.setBlendMode(.copy)
        context.draw(_layer, at: .zero)

        // Restore to the previous state
        context.restoreGState()
    }
}
