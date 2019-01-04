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
public class FloView : NSView {
    /// The view that this will display
    fileprivate var _view: NSView!;
    
    /// The layout bounds of this view
    fileprivate var _bounds: Bounds;
    
    required init?(coder: NSCoder) {
        _bounds = Bounds(
            x1: Position.Start,
            y1: Position.Start,
            x2: Position.End,
            y2: Position.End
        );
        
        super.init(coder: coder);
    }
    
    override init(frame: NSRect) {
        _bounds = Bounds(
            x1: Position.Start,
            y1: Position.Start,
            x2: Position.End,
            y2: Position.End
        );
        
        super.init(frame: frame);
        
        self.wantsLayer                             = true;
    }
    
    override public var isOpaque: Bool { get { return false; } }
    
    ///
    /// The bounds of this view
    ///
    internal var floBounds: Bounds {
        get { return _bounds; }
    }

    ///
    /// The view that this is managing
    ///
    public var view: NSView! {
        get { return self; }
    }
    
    override public func setFrameSize(_ newSize: NSSize) {
        super.setFrameSize(newSize);
        self.performLayout();
    }
    
    ///
    /// Performs layout of this view immediately
    ///
    public func performLayout() {
        // Just pass the request on to the layout class
        Layout.layoutView(view: self);
    }
    
    ///
    /// Creates an empty view
    ///
    @objc public func setupAsEmpty() {
        self.layer!.backgroundColor = NSColor.init(deviceRed: 1.0, green: 1.0, blue: 1.0, alpha: 0.0).cgColor;
    }
    
    ///
    /// Removes this view from its superview
    ///
    @objc public func viewRemoveFromSuperview() {
        _view?.removeFromSuperview();
    }
    
    ///
    /// Adds a subview to this view
    ///
    @objc(viewAddSubView:) public func viewAddSubView(subview: FloView!) {
        if let subview = subview.view {
            self.addSubview(subview);
        }
    }
    
    ///
    /// Sets the position of a side of the view
    ///
    func set_side_position(_ side: Int32, _ position: Position) {
        switch (side) {
        case 0: _bounds.x1 = position;
        case 1: _bounds.y1 = position;
        case 2: _bounds.x2 = position;
        case 3: _bounds.y2 = position;
        default: break;
        }
    }
    
    @objc(viewSetSide:at:) public func viewSetSide(side: Int32, at: Float32) {
        set_side_position(side, Position.At(at));
    }

    @objc(viewSetSide:offset:) public func viewSetSide(side: Int32, offset: Float32) {
        set_side_position(side, Position.Offset(offset));
    }

    @objc(viewSetSide:stretch:) public func viewSetSide(side: Int32, stretch: Float32) {
        set_side_position(side, Position.Stretch(stretch));
    }

    @objc(viewSetSideAtStart:) public func viewSetSideAtStart(side: Int32) {
        set_side_position(side, Position.Start);
    }

    @objc(viewSetSideAtEnd:) public func viewSetSideAtEnd(side: Int32) {
        set_side_position(side, Position.End);
    }

    @objc(viewSetSideAfter:) public func viewSetSideAfter(side: Int32) {
        set_side_position(side, Position.After);
    }
    
    ///
    /// Sets the z-ordering of this view
    ///
    @objc public func viewSetZIndex(_ zIndex: Float64) {
        self.layer?.zPosition = CGFloat(zIndex);
    }
    
    ///
    /// Sets the foreground (text) colour of the view
    ///
    @objc public func viewSetForegroundRed(_ red: Float64, green: Float64, blue: Float64, alpha: Float64) {
        let col = NSColor(calibratedRed: CGFloat(red), green: CGFloat(green), blue: CGFloat(blue), alpha: CGFloat(alpha));
    }

    ///
    /// Sets the background colour of the view
    ///
    @objc public func viewSetBackgroundRed(_ red: Float64, green: Float64, blue: Float64, alpha: Float64) {
        let col = NSColor(calibratedRed: CGFloat(red), green: CGFloat(green), blue: CGFloat(blue), alpha: CGFloat(alpha));
        
        self.layer?.backgroundColor = col.cgColor;
    }
}
