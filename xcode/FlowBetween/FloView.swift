//
//  FloView.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 03/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Foundation
import Cocoa

///
/// Class used to manage a view in FlowBetween
///
public class FloView : NSObject, FloViewDelegate {
    /// The view that this represents
    fileprivate var _view: FloContainerView;
    
    /// The view that this is a subview of
    fileprivate weak var _superview: FloView?;
    
    /// The layout bounds of this view
    fileprivate var _bounds: Bounds;
    
    /// The padding set for this view
    fileprivate var _padding: Padding?;
    
    /// The subviews of this view
    fileprivate var _subviews: [FloView];
    
    /// Set to true if we've queued up a relayout operation
    fileprivate var _willLayout: Bool = false;
    
    /// Events
    fileprivate var _onClick: (() -> ())?;
    
    /// The layer to draw on, if there is one
    fileprivate var _drawingLayer: FloCanvasLayer?;
    
    override init() {
        _bounds = Bounds(
            x1: Position.Start,
            y1: Position.Start,
            x2: Position.End,
            y2: Position.End
        );
        _subviews = [];

        _view = FloEmptyView.init();

        super.init();

        weak var this = self;
        
        _view.performLayout = { size in if let this = this { this.performLayout(size) } };
        _view.onClick       = { if let onClick = this?._onClick { onClick(); return true; } else { return false; } }
    }
    
    required init(withView view: FloContainerView) {
        _bounds = Bounds(
            x1: Position.Start,
            y1: Position.Start,
            x2: Position.End,
            y2: Position.End
        );
        _subviews = [];
        
        _view = view;
        
        super.init();
        
        weak var this = self;
        
        _view.performLayout = { size in if let this = this { this.performLayout(size) } };
        _view.onClick       = { if let onClick = this?._onClick { onClick(); return true; } else { return false; } }
    }
    
    ///
    /// The bounds of this view, as described to the layout system
    ///
    internal var floBounds: Bounds {
        get { return _bounds; }
    }
    
    ///
    /// The padding for this view (extra space used after layout)
    ///
    internal var floPadding: Padding? {
        get { return _padding; }
    }

    ///
    /// The view that this is managing
    ///
    public var view: NSView! {
        get { return _view.asView; }
    }
    
    ///
    /// The subviews that should be laid out within this view
    ///
    public var layoutSubviews: [FloView] {
        get {
            return _subviews;
        }
    }
    
    ///
    /// Performs layout of this view immediately
    ///
    public func performLayout(_ size: NSSize) {
        // Just pass the request on to the layout class
        FloLayout.layoutView(view: self, size: size, state: _view.viewState);
    }
    
    ///
    /// Invalidates the layout of this view
    ///
    public func invalidateLayout() {
        if !_willLayout {
            _willLayout = true;
            
            RunLoop.main.perform(inModes: [RunLoop.Mode.default, RunLoop.Mode.eventTracking], block: {
                self._willLayout = false;
                self.performLayout(self._view.layoutSize);
            });
        }
    }
    
    ///
    /// Performs the click event/action for this view (callback for controls)
    ///
    @objc func onClick() {
        _view.triggerClick();
    }
    
    ///
    /// Sends an event if this view (or its control) is clicked
    ///
    @objc public func requestClick(_ events: FloEvents, withName: String?) {
        _onClick = { events.sendClick(withName); };
    }
    
    @objc public func requestDismiss(_ events: FloEvents!, withName name: String!) {
        NSLog("RequestDismiss not implemented");
    }
    
    @objc public func requestDrag(_ events: FloEvents!, withName name: String!) {
        NSLog("RequestDrag not implemented");
    }
    
    @objc public func requestFocused(_ events: FloEvents!, withName name: String!) {
        _view.onFocused = { events.sendFocus(name); };
    }
    
    @objc public func requestEditValue(_ events: FloEvents!, withName name: String!) {
        _view.onEditValue = { propertyValue in
            switch propertyValue {
            case .Bool(let boolVal):    events.sendChangeValue(name, isSet: false, with: boolVal); break;
            case .Float(let floatVal):  events.sendChangeValue(name, isSet: false, with: floatVal); break;
            case .Int(let intVal):      events.sendChangeValue(name, isSet: false, with: Double(intVal)); break;
            case .String(let strVal):   events.sendChangeValue(name, isSet: false, with: strVal); break;
            default:                    break;
            }
        }
    }
    
    @objc public func requestSetValue(_ events: FloEvents!, withName name: String!) {
        _view.onSetValue = { propertyValue in
            switch propertyValue {
            case .Bool(let boolVal):    events.sendChangeValue(name, isSet: true, with: boolVal); break;
            case .Float(let floatVal):  events.sendChangeValue(name, isSet: true, with: floatVal); break;
            case .Int(let intVal):      events.sendChangeValue(name, isSet: true, with: Double(intVal)); break;
            case .String(let strVal):   events.sendChangeValue(name, isSet: true, with: strVal); break;
            default:                    break;
            }
        }
    }
    
    @objc public func requestCancelEdit(_ events: FloEvents!, withName name: String!) {
        NSLog("RequestCancelEdit not implemented");
    }
    
    @objc public func viewSetSelected(_ property: FloProperty!) {
        _view.setState(selector: ViewStateSelector.Selected, toProperty: property);
    }
    
    @objc public func viewSetBadged(_ property: FloProperty!) {
        _view.setState(selector: ViewStateSelector.Badged, toProperty: property);
    }
    
    @objc public func viewSetEnabled(_ property: FloProperty!) {
        _view.setState(selector: ViewStateSelector.Enabled, toProperty: property);
    }
    
    @objc public func viewSetValue(_ property: FloProperty!) {
        _view.setState(selector: ViewStateSelector.Value, toProperty: property);
    }
    
    @objc public func viewSetRange(withLower lower: FloProperty!, upper: FloProperty!) {
        _view.setState(selector: ViewStateSelector.RangeLower, toProperty: lower);
        _view.setState(selector: ViewStateSelector.RangeHigher, toProperty: upper);
    }
    
    @objc public func viewSetFocusPriority(_ property: FloProperty!) {
        _view.setState(selector: ViewStateSelector.FocusPriority, toProperty: property);
    }
    
    ///
    /// Sends an event if this view is scrolled
    ///
    @objc public func requestVirtualScroll(_ events: FloEvents, withName: String?, width scrollWidth: Float64, height scrollHeight: Float64) {
        var (x, y)          = (UInt32(0), UInt32(0));
        var (width, height) = (UInt32(0), UInt32(0));
        
        _view.onScroll = { visibleRect in
            let (newXf, newYf)          = (Float64(visibleRect.minX) / scrollWidth, Float64(visibleRect.minY) / scrollHeight);
            let (newXi, newYi)          = (UInt32(floor(Float64.maximum(newXf, 0))), UInt32(floor(Float64.maximum(newYf, 0))));
            
            let (newWidthf, newHeightf) = (Float64(visibleRect.width) / scrollWidth, Float64(visibleRect.height)/scrollHeight);
            let (newWidthi, newHeighti) = (UInt32(floor(newWidthf)+1.0), UInt32(floor(newHeightf)+1.0));
            
            if newXi != x || newYi != y || newWidthi != width || newHeighti != height {
                x       = newXi;
                y       = newYi;
                width   = newWidthi;
                height  = newHeighti;
                
                events.sendVirtualScroll(withName!, left: newXi, top: newYi, width: width, height: height);
            }
        };
    }
    
    ///
    /// Sends an event when the user uses the specified device to paint on this view
    ///
    @objc public func requestPaint(withDeviceId deviceId: UInt32, events: FloEvents, withName: String?) {
        // Convert the device ID into the device enum
        let device = FloPaintDevice.init(rawValue: deviceId);
        
        if let device = device {
            // Ask the underlying view to relay paint events to us
            _view.onPaint[device] = { stage, painting in
                switch (stage) {
                case FloPaintStage.Start:       events.sendPaintStart(forDevice: deviceId, name: withName, action: painting); break;
                case FloPaintStage.Continue:    events.sendPaintContinue(forDevice: deviceId, name: withName, action: painting); break;
                case FloPaintStage.Finish:      events.sendPaintFinish(forDevice: deviceId, name: withName, action: painting); break;
                case FloPaintStage.Cancel:      events.sendPaintCancel(forDevice: deviceId, name: withName, action: painting); break;
                }
                
            }
        } else {
            // Device not in the enum
            NSLog("Unknown paint device ID \(deviceId)");
        }
    }
    
    ///
    /// Removes this view from its superview
    ///
    @objc public func viewRemoveFromSuperview() {
        // Remove the view from the view hierarchy
        _view.asView.removeFromSuperview();
        
        // Remove from its parent FloView
        if let superview = _superview {
            superview._subviews.removeAll(where: { view in return view == self });
        }
    }
    
    ///
    /// Adds a subview to this view
    ///
    @objc(viewAddSubView:) public func viewAddSubView(_ subview: NSObject) {
        let subview = subview as! FloView;
        subview.viewRemoveFromSuperview();
        
        _subviews.append(subview);
        subview._superview = self;
        
        if let subview = subview.view {
            _view.addContainerSubview(subview);
        }
        
        // View will need to be laid out again
        invalidateLayout();
    }
    
    ///
    /// Inserts a subview in a particular place in the list of subviews of this view
    ///
    @objc public func viewInsertSubView(_ subview: NSObject!, at index: UInt32) {
        let subview = subview as! FloView;
        subview.viewRemoveFromSuperview();
        
        _subviews.insert(subview, at: Int(index));
        subview._superview = self;
        
        if let subview = subview.view {
            _view.addContainerSubview(subview);
        }
        
        // View will need to be laid out again
        invalidateLayout();
    }

    ///
    /// Sets the position of a side of the view
    ///
    func setSidePosition(_ side: Int32, _ position: Position) {
        switch (side) {
        case 0: _bounds.x1 = position;
        case 1: _bounds.y1 = position;
        case 2: _bounds.x2 = position;
        case 3: _bounds.y2 = position;
        default: break;
        }
    }
    
    @objc(viewSetSide:at:) public func viewSetSide(_ side: Int32, at: Float64) {
        setSidePosition(side, Position.At(at));
    }

    @objc(viewSetSide:offset:) public func viewSetSide(_ side: Int32, offset: Float64) {
        setSidePosition(side, Position.Offset(offset));
    }

    @objc public func viewSetSide(_ side: Int32, offset: Float64, floating floatingOffset: FloProperty!) {
        setSidePosition(side, Position.Floating(offset, floatingOffset));
    }
    

    @objc(viewSetSide:stretch:) public func viewSetSide(_ side: Int32, stretch: Float64) {
        setSidePosition(side, Position.Stretch(stretch));
    }

    @objc(viewSetSideAtStart:) public func viewSetSide(atStart side: Int32) {
        setSidePosition(side, Position.Start);
    }

    @objc(viewSetSideAtEnd:) public func viewSetSide(atEnd side: Int32) {
        setSidePosition(side, Position.End);
    }

    @objc(viewSetSideAfter:) public func viewSetSide(after side: Int32) {
        setSidePosition(side, Position.After);
    }
    
    ///
    /// Sets the padding around this view
    ///
    @objc public func viewSetPadding(withLeft left: Double, top: Double, right: Double, bottom: Double) {
        _padding = Padding(left: left, top: top, right: right, bottom: bottom);
    }

    ///
    /// Sets the z-ordering of this view
    ///
    @objc public func viewSetZIndex(_ zIndex: Float64) {
        _view.asView.layer?.zPosition = CGFloat(zIndex);
    }
    
    ///
    /// Sets the foreground (text) colour of the view
    ///
    @objc public func viewSetForegroundRed(_ red: Float64, green: Float64, blue: Float64, alpha: Float64) {
        let col = NSColor(calibratedRed: CGFloat(red), green: CGFloat(green), blue: CGFloat(blue), alpha: CGFloat(alpha));
        
        _view.setForegroundColor(color: col);
    }

    ///
    /// Sets the background colour of the view
    ///
    @objc public func viewSetBackgroundRed(_ red: Float64, green: Float64, blue: Float64, alpha: Float64) {
        let col = NSColor(calibratedRed: CGFloat(red), green: CGFloat(green), blue: CGFloat(blue), alpha: CGFloat(alpha));
        
        _view.asView.layer?.backgroundColor = col.cgColor;
    }
    
    var _text: FloProperty?;
    ///
    /// Sets the text for the view
    ///
    @objc public func viewSetText(_ text: FloProperty) {
        _text           = text;
        weak var this   = self;
        
        text.trackValue({ value in
            if case let PropertyValue.String(value) = value {
                this?._view.setTextLabel(label: value);
            }
        });
    }
    
    var _image: NSImage?;
    var _imageView: NSView?;
    var _imageLayer: CALayer?;
    var _imageResolution: CGFloat = 1.0;
    
    ///
    /// Sets the position of the image view within the main view
    ///
    func repositionImageView(_ bounds: ContainerBounds) {
        if let imageLayer = _imageLayer {
            if let screen = _view.asView.window?.screen {
                // Work out how to scale the image (so that it can be displayed by the view)
                let resolution  = screen.backingScaleFactor;

                // Reset the image contents
                if resolution != _imageResolution {
                    _imageResolution = resolution;
                    imageLayer.contentsScale    = resolution;
                    imageLayer.contentsGravity  = CALayerContentsGravity.resizeAspect;
                    imageLayer.contents         = _image?.layerContents(forContentsScale: resolution);
                }
            }
        }
    }
    
    ///
    /// Sets the image for the view
    ///
    @objc public func viewSetImage(_ image: NSImage) {
        _image = image;
        
        // Add an image view to this view if one does not already exist
        if _imageView == nil {
            _imageView              = NSView.init();
            _imageView!.wantsLayer  = true;
            
            _imageLayer             = _imageView!.layer!;
            
            _view.addContainerSubview(_imageView!);
        }
        
        // Update the image when the view bounds change
        _imageResolution = 0.0;
        
        weak var this = self;
        _view.boundsChanged = { newBounds in
            this?.repositionImageView(newBounds);
        };
        _view.triggerBoundsChanged();
    }
    
    ///
    /// Sets the font size of the control for this view
    ///
    @objc public func viewSetFontSize(_ size: Float64) {
        _view.setFontSize(points: size);
    }
    
    ///
    /// Sets the font weight of the control for this view
    ///
    @objc public func viewSetFontWeight(_ weight: Float64) {
        _view.setFontWeight(weight: weight);
    }

    ///
    /// Sets the text alignment of the control for this view
    ///
    /// Alignments are:
    ///     0 - Left
    ///     1 - Center
    ///     2 - Right
    ///
    @objc public func viewSetTextAlignment(_ alignment: UInt32) {
        switch alignment {
        case 0:     _view.setTextAlignment(alignment: NSTextAlignment.left);    break;
        case 1:     _view.setTextAlignment(alignment: NSTextAlignment.center);  break;
        case 2:     _view.setTextAlignment(alignment: NSTextAlignment.right);   break;
        default:    break;
        }
    }
    
    ///
    /// Sets the minimum size for the scrollable area of this view
    ///
    @objc(viewSetScrollMinimumSizeWithWidth:height:) public func viewSetScrollMinimumSize(withWidth width: Float64, height: Float64) {
        _view.scrollMinimumSize = (width, height);
    }
    
    func getScrollBarVisibility(_ intVisibility: UInt32) -> ScrollBarVisibility {
        switch (intVisibility) {
        case 0:     return ScrollBarVisibility.Never;
        case 1:     return ScrollBarVisibility.Always;
        case 2:     return ScrollBarVisibility.OnlyIfNeeded;
        default:    return ScrollBarVisibility.OnlyIfNeeded;
        }
    }
    
    ///
    /// Sets the horizontal scroll bar visibility
    ///
    @objc public func viewSetHorizontalScrollVisibility(_ visibility: UInt32) {
        let (_, vert) = _view.scrollBarVisibility;
        _view.scrollBarVisibility = (getScrollBarVisibility(visibility), vert);
    }

    ///
    /// Sets the horizontal scroll bar visibility
    ///
    @objc public func viewSetVerticalScrollVisibility(_ visibility: UInt32) {
        let (horiz, _) = _view.scrollBarVisibility;
        _view.scrollBarVisibility = (horiz, getScrollBarVisibility(visibility));
    }
    
    ///
    /// Updates the bounds of the drawing layer (and its context) after the
    ///
    func drawingLayerBoundsChanged(_ newBounds: ContainerBounds) {
        autoreleasepool {
            let layer = _drawingLayer!;
            
            // Work out the screen resolution of the current window
            var resolutionMultiplier = CGFloat(1.0);
            if let window = _view.asView.window {
                if let screen = window.screen {
                    resolutionMultiplier = screen.backingScaleFactor;
                }
            }
            
            // Perform the action instantly rather than with the default animation
            CATransaction.begin();
            CATransaction.setAnimationDuration(0.0);
            CATransaction.setDisableActions(true);

            // Move the layer so that it fills the visible bounds of the view
            let parentBounds    = _view.asView.layer!.bounds;
            var visibleRect     = newBounds.visibleRect;
            
            visibleRect.origin.x += parentBounds.origin.x;
            visibleRect.origin.y += parentBounds.origin.y;
            if visibleRect.size.width < 1.0 { visibleRect.size.width = 1.0; }
            if visibleRect.size.height < 1.0 { visibleRect.size.height = 1.0; }
            
            layer.frame         = visibleRect;
            
            CATransaction.commit();
            
            // Regenerate the graphics context so that it's the appropriate size for the layer
            _drawingLayer?.setVisibleArea(bounds: newBounds, resolution: resolutionMultiplier);
            
            redisplayCanvasLayer();
        }
    }
    
    ///
    /// Creates the layer that will be used to draw canvas items for this view
    ///
    func createCanvasDrawingLayer(_ events: FloEvents) {
        // Create the layer
        let layer       = FloCanvasLayer();
        
        // Layer should not animate its contents
        layer.actions = [
            "onOrderIn":    NSNull(),
            "onOrderOut":   NSNull(),
            "sublayers":    NSNull(),
            "contents":     NSNull(),
            "bounds":       NSNull(),
            "frame":        NSNull()
        ];
        
        _drawingLayer = layer;
        
        // Reset the layer size when the bounds change
        weak var this = self;
        var willChangeBounds = false;
        _view.boundsChanged = { newBounds in
            if !willChangeBounds {
                willChangeBounds = true;
                
                RunLoop.main.perform(inModes: [RunLoop.Mode.default, RunLoop.Mode.eventTracking], block: {
                    willChangeBounds = false;
                    this?.drawingLayerBoundsChanged(newBounds);
                });
            }
        }
        
        var initialSize = _view.layoutSize;
        if initialSize.width < 1 { initialSize.width = 1 }
        if initialSize.height < 1 { initialSize.height = 1 }
        
        layer.onRedraw              { (canvasSize, viewport) in events.redrawCanvas(with: canvasSize, viewport: viewport); }
        layer.backgroundColor       = CGColor.clear;
        layer.frame                 = CGRect(x: 0, y: 0, width: initialSize.width, height: initialSize.height);
        layer.drawsAsynchronously  = true;
        layer.setNeedsDisplay();
        
        RunLoop.main.perform(inModes: [RunLoop.Mode.default, RunLoop.Mode.modalPanel, RunLoop.Mode.eventTracking], block: { self._view.setCanvasLayer(layer) });
    }

    var _willRedisplayCanvasLayer = false;
    ///
    /// Causes the canvas layer to be redisplayed need time through the runloop
    ///
    func redisplayCanvasLayer() {
        if !_willRedisplayCanvasLayer {
            _willRedisplayCanvasLayer = true;
            RunLoop.main.perform(inModes: [RunLoop.Mode.default, RunLoop.Mode.modalPanel, RunLoop.Mode.eventTracking], block: {
                CATransaction.begin();
                CATransaction.setAnimationDuration(0.0);
                CATransaction.setDisableActions(true);

                self._willRedisplayCanvasLayer = false;
                self._drawingLayer?.setNeedsDisplay();
                self._drawingLayer?.display();
                
                CATransaction.commit();
            });
        }
    }
    
    ///
    /// Retrieves the drawing context for this view
    ///
    @objc public func viewGetCanvas(forDrawing events: FloEvents, layer: UInt32) -> Unmanaged<CGContext>? {
        // Create the drawing layer if one doesn't exist yet
        if _drawingLayer == nil {
            createCanvasDrawingLayer(events);
        }
        
        // Make sure the backing for the layer has been created
        if let context = _drawingLayer?.getContextForLayer(id: layer) {
            return Unmanaged.passUnretained(context);
        } else {
            return nil;
        }
    }
    
    ///
    /// Copies the contents of a particular layer in the canvas
    ///
    @objc public func viewCopyLayer(withId layerId: UInt32) -> FloCacheLayer? {
        return _drawingLayer?.cacheLayerWithId(id: layerId)
    }
    
    ///
    /// Restores an existing layer from a cached layer
    ///
    @objc public func viewRestoreLayer(to layerId: UInt32, fromCopy: FloCacheLayer?) {
        if let fromCopy = fromCopy {
            _drawingLayer?.restoreLayerFromCache(id: layerId, cachedCopy: fromCopy);
        }
    }
    
    var _willUpdateCanvas = false;
    ///
    /// Drawing on the context has finished
    ///
    @objc public func viewFinishedDrawing() {
        redisplayCanvasLayer();
    }
    
    ///
    /// Sets the transform for any mouse clicks, etc for this view
    ///
    @objc public func viewSetTransform(_ transform: CGAffineTransform) {
        _view.canvasAffineTransform = transform;
    }
    
    ///
    /// The drawing canvas should be entirely cleared
    ///
    @objc public func viewClearCanvas() {
        _drawingLayer?.clearBackingLayers();
    }
}
