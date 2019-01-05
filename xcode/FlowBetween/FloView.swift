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
    /// The control contained by this view
    fileprivate var _control: NSControl!;
    
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
    
    ///
    /// The subviews that should be laid out within this view
    ///
    public var layoutSubviews: [NSView] {
        get {
            return self.subviews;
        }
    }
    
    ///
    /// The bounds within which the subviews should be laid out
    ///
    public var layoutBounds: NSRect {
        get {
            return self.bounds;
        }
    }
    
    public var control: NSControl {
        get {
            if let control = _control {
                // Use the existing control if there is one
                return control;
            } else {
                // Default control is a label
                let label   = NSTextField.init(labelWithString: "");
                label.font  = NSFontManager.shared.font(withFamily: "Lato", traits: NSFontTraitMask(), weight: 5, size: 13.0);
                
                self.addSubview(label);
                _control = label;

                return label;
            }
        }
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
    /// Creates an empty view
    ///
    @objc public func setupAsButton() {
        let button = NSButton.init(title: "", target: nil, action: nil);
        
        self.addSubview(button);
        _control = button;
    }

    ///
    /// Removes this view from its superview
    ///
    @objc public func viewRemoveFromSuperview() {
        self.removeFromSuperview();
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
        
        // TODO: need to support attributed strings :-/
    }

    ///
    /// Sets the background colour of the view
    ///
    @objc public func viewSetBackgroundRed(_ red: Float64, green: Float64, blue: Float64, alpha: Float64) {
        let col = NSColor(calibratedRed: CGFloat(red), green: CGFloat(green), blue: CGFloat(blue), alpha: CGFloat(alpha));
        
        self.layer?.backgroundColor = col.cgColor;
    }
    
    ///
    /// Sets the text for the view
    ///
    @objc public func viewSetText(_ text: FloProperty) {
        if case let PropertyValue.String(value) = text.value {
            control.stringValue = value;
        }
    }
    
    var _imageView: NSImageView!;
    
    ///
    /// Sets the image for the view
    ///
    @objc public func viewSetImage(_ image: NSImage) {
        // Add an image view to this view if one does not already exist
        if _imageView == nil {
            _imageView = NSImageView.init();
            self.addSubview(_imageView);
        }
        
        // Change its image
        _imageView!.image = image;
    }
    
    ///
    /// Sets the font size of the control for this view
    ///
    @objc public func viewSetFontSize(_ size: Float64) {
        let existingFont    = control.font!;
        let newFont         = NSFontManager.shared.convert(existingFont, toSize: CGFloat(size));
        
        control.font        = newFont;
    }
    
    ///
    /// Converts a weight from a value like 100, 200, 400, etc to a font manager weight (0-15)
    ///
    func convertWeight(_ weight: Float64) -> Int {
        if weight <= 150.0 {
            return 1;
        } else if weight <= 450.0 {
            return 5;
        } else if weight <= 750.0 {
            return 7;
        } else {
            return 10;
        }
    }
    
    ///
    /// Sets the font weight of the control for this view
    ///
    @objc public func viewSetFontWeight(_ weight: Float64) {
        let existingFont        = control.font!;
        let fontManagerWeight   = convertWeight(weight);
        let family              = existingFont.familyName!;
        let size                = existingFont.pointSize;
        let traits              = NSFontTraitMask();
        
        let newFont             = NSFontManager.shared.font(withFamily: family, traits: traits, weight: fontManagerWeight, size: size);
        
        control.font        = newFont;
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
        case 0:     control.alignment = NSTextAlignment.left;   break;
        case 1:     control.alignment = NSTextAlignment.center; break;
        case 2:     control.alignment = NSTextAlignment.right;  break;
        default:    break;
        }
    }
}
