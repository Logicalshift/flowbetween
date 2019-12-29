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
    /// The number of times this layer has been cleared (so we don't return backing layers)
    fileprivate var _clearCount: UInt32 = 0

    /// The backing for this layer (nil if it's not drawable yet)
    fileprivate var _backing: [UInt32: CGLayer]

    /// Layers that we stopped using during the last clear command
    fileprivate var _unusedLayers: [CGLayer]

    /// Function called to trigger a redraw
    fileprivate var _triggerRedraw: ((NSSize, NSRect) -> ())?

    /// The overall size of the canvas
    fileprivate var _canvasSize: NSSize

    /// The coordinates of the visible region in the canvsa
    fileprivate var _visibleRect: NSRect

    /// The resolution of this layer
    fileprivate var _resolution: CGFloat = 1.0

    var canvasSize: CGSize { return _canvasSize }

    override init() {
        _canvasSize     = NSSize(width: 1, height: 1)
        _visibleRect    = NSRect(x: 0, y: 0, width: 1, height: 1)
        _backing        = [UInt32: CGLayer]()
        _unusedLayers   = []

        super.init()
    }

    override init(layer: Any) {
        _canvasSize     = NSSize(width: 1, height: 1)
        _visibleRect    = NSRect(x: 0, y: 0, width: 1, height: 1)
        _backing        = [UInt32: CGLayer]()
        _unusedLayers   = []

        super.init()

        if let layer = layer as? FloCanvasLayer {
            _backing            = layer._backing
            _canvasSize         = layer._canvasSize
            _visibleRect        = layer._visibleRect
            _resolution         = layer._resolution
        }
    }

    required init?(coder aDecoder: NSCoder) {
        _canvasSize     = NSSize(width: 1, height: 1)
        _visibleRect    = NSRect(x: 0, y: 0, width: 1, height: 1)
        _backing        = [UInt32: CGLayer]()
        _unusedLayers   = []

        super.init(coder: aDecoder)
    }

    ///
    /// Draws this layer in response
    ///
    override func draw(in ctx: CGContext) {
        // Redraw the backing layer if it has been invalidated
        if _backing.count == 0 {
            // TODO: for whatever reason the first layer we generate is 'bad' and renders slowly
            // Appears that regenerating the layer instead of caching it in the unused set fixes this issue
            // (Eg: resize the view)

            var size    = _visibleRect.size
            size.width  *= _resolution
            size.height *= _resolution

            if size.width <= 0  { size.width = 1 }
            if size.height <= 0 { size.height = 1 }

            // Create the backing layer (there's always a layer 0 by default)
            _backing[0] = CGLayer(ctx, size: size, auxiliaryInfo: nil)

            if _resolution != 1.0 {
                let scale = CGAffineTransform(scaleX: _resolution, y: _resolution)
                _backing[0]!.context!.concatenate(scale)
            }

            // Force a redraw via the events
            autoreleasepool { _triggerRedraw?(_canvasSize, _visibleRect) }
        }

        // Draw the backing layer on this layer
        let layer_ids   = _backing.keys.sorted()
        let bounds      = self.bounds

        ctx.saveGState()
        ctx.setShouldAntialias(false)
        ctx.interpolationQuality = .none
        if _resolution != 1.0 {
            ctx.concatenate(CGAffineTransform(scaleX: 1.0/_resolution, y: 1.0/_resolution))
        }

        for layer_id in layer_ids {
            ctx.draw(_backing[layer_id]!, at: bounds.origin)
        }

        ctx.restoreGState()
    }

    ///
    /// Updates the area of the canvas that this layer should display
    ///
    func setVisibleArea(bounds: ContainerBounds, resolution: CGFloat) {
        autoreleasepool {
            if _visibleRect.size != bounds.visibleRect.size || resolution != _resolution {
                // Backing will have changed size, so invalidate it entirely
                invalidateAllLayers()
                setNeedsDisplay()
            } else {
                // Just trigger a redraw
                _triggerRedraw?(bounds.totalSize, bounds.visibleRect)
            }

            _canvasSize         = bounds.totalSize
            _visibleRect        = bounds.visibleRect

            _resolution         = resolution
            contentsScale       = resolution

            CATransaction.begin()
            CATransaction.setAnimationDuration(0.0)
            CATransaction.disableActions()
            displayIfNeeded()
            CATransaction.commit()
        }
    }

    ///
    /// Sets the function to call when the layer needs to be redrawn
    ///
    func onRedraw(_ redraw: @escaping ((NSSize, NSRect) -> ())) {
        _triggerRedraw = redraw
    }

    ///
    /// Clears the backing layers for this layer
    ///
    func clearBackingLayers() {
        // All layers other than layer 0 are removed (pushed onto the unused layer list)
        let layers_to_remove = _backing.keys.filter({ layer_id in layer_id != 0 })
        for layer_id in layers_to_remove {
            _unusedLayers.append(_backing[layer_id]!)
            _backing.removeValue(forKey: layer_id)
        }

        // Clear the bottom layer
        _backing[0]?.context?.clear(CGRect(size: self.bounds.size))
    }

    ///
    /// Ensures the layer with the specifed ID exists
    ///
    func getContextForLayer(id: UInt32) -> CGContext? {
        if _backing.keys.contains(id) {
            // Layer already exists
            return _backing[id]?.context
        } else if let availableLayer = _unusedLayers.popLast() {
            // Use a layer we created earlier if we can
            _backing[id] = availableLayer

            // Make sure it has nothing already rendered on it
            availableLayer.context?.clear(CGRect(size: _visibleRect.size))
            return availableLayer.context
        } else if let baseLayer = _backing[0] {
            // Get the size for the new layer
            var size    = _visibleRect.size
            size.width  *= _resolution
            size.height *= _resolution

            if size.width <= 0 { size.width = 1 }
            if size.height <= 0 { size.height = 1 }

            // We create the new layer from a base layer (as CGLayer needs a context to work from)
            let newLayer = CGLayer(baseLayer.context!, size: size, auxiliaryInfo: nil)

            if _resolution != 1.0 {
                let scale = CGAffineTransform(scaleX: _resolution, y: _resolution)
                newLayer!.context!.concatenate(scale)
            }

            // Store the new layer as a new backing layer
            _backing[id] = newLayer!
            return newLayer?.context
        } else {
            // No base layer, so we can't create new layers
            return nil
        }
    }

    ///
    /// Invalidates all of the layers in this object.
    ///
    /// This will remove them entirely: normally when the canvas is cleared we keep track of
    /// any layers we were using before so we don't need to reallocate them in the event of
    /// a redraw. However, this will produce invalid results when the layer is resized.
    ///
    func invalidateAllLayers() {
        // Both the backing and the unused layers become invalidated so we can't re-use them
        _backing        = [UInt32: CGLayer]()
        _unusedLayers   = []
        _clearCount     += 1
    }

    ///
    /// A CGLayer created for this layer has become unused and is being returned to the cache list
    ///
    func returnUnusedLayer(_ layer: CGLayer, _ clearCount: UInt32) {
        if _clearCount == clearCount {
            _unusedLayers.append(layer)
        }
    }

    ///
    /// Creates a cached copy of the layer with the specified ID
    ///
    func cacheLayerWithId(id: UInt32) -> FloCacheLayer? {
        let cacheLayer: FloCacheLayer

        if let availableLayer = _unusedLayers.popLast() {
            // Use an unused layer if there is one
            cacheLayer = FloCacheLayer(layer: availableLayer, canvas: self, clearCount: _clearCount)
        } else if let baseLayer = _backing[0] {
            // Create a new layer if there is none avialable
            // Get the size for the new layer
            var size    = _visibleRect.size
            size.width  *= _resolution
            size.height *= _resolution

            if size.width == 0 { size.width = 1 }
            if size.height == 0 { size.height = 1 }

            // We create the new layer from a base layer (as CGLayer needs a context to work from)
            let newLayer = CGLayer(baseLayer.context!, size: size, auxiliaryInfo: nil)

            if _resolution != 1.0 {
                let scale = CGAffineTransform(scaleX: _resolution, y: _resolution)
                newLayer!.context!.concatenate(scale)
            }

            cacheLayer = FloCacheLayer(layer: newLayer!, canvas: self, clearCount: _clearCount)
        } else {
            // If there's no backing layer, there's nowhere to create a cache layer
            return nil
        }

        if let cacheFrom = _backing[id] {
            cacheLayer.cache(from: cacheFrom)
        }

        // The new cache layer is the result
        return cacheLayer
    }

    ///
    /// Updates an already cached layer
    ///
    func updateCachedLayer(_ layer: FloCacheLayer, id: UInt32) {
        if let cacheFrom = _backing[id] {
            layer.cache(from: cacheFrom)
        }
    }

    ///
    /// Restores a cached layer to another layer
    ///
    func restoreLayerFromCache(id: UInt32, cachedCopy: FloCacheLayer) {
        if let restoreLayer = _backing[id] {
            cachedCopy.restore(to: restoreLayer)
        }
    }
}
