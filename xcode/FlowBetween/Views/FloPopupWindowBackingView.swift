//
//  FloPopupWindowBackingVIew.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 13/02/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

///
/// The backing view for a popup window
///
class FloPopupWindowBackingView : NSView {
    fileprivate let _borderWidth = CGFloat(8.0)
    fileprivate let _beakWidth = CGFloat(24.0)
    fileprivate let _beakHeight = CGFloat(12.0)

    override func setFrameSize(_ newSize: NSSize) {
        super.setFrameSize(newSize)

        // Subview bounds depend on the direction
        var subviewBounds = CGRect(size: newSize).insetBy(dx: _borderWidth, dy: _borderWidth)

        if newSize.width < _borderWidth*2 || newSize.height < _borderWidth*2 {
            subviewBounds = .zero
        }

        switch (direction) {
        case .Above:
            subviewBounds.origin.y += _beakHeight; subviewBounds.size.height -= _beakHeight
        case .Below:
            subviewBounds.size.height -= _beakHeight
        case .Left:
            subviewBounds.origin.x += _beakHeight; subviewBounds.size.width -= _beakHeight
        case .Right:
            subviewBounds.size.width -= _beakHeight

        case .OnTop, .WindowCentered, .WindowTop:
            break
        }

        // Update the bounds of all the subviews
        subviews.forEach { subview in
            subview.frame = subviewBounds
        }
    }

    /// The popup direction for this view
    var direction: PopupDirection = .Above

    ///
    /// Calulates the required size for this view given a content size and a direction
    ///
    func sizeForContentSize(_ contentSize: NSSize, _ direction: PopupDirection) -> NSSize {
        switch (direction) {
        case .Above:            return NSSize(width: contentSize.width + _borderWidth*2, height: contentSize.height + _borderWidth*2 + _beakHeight)
        case .Below:            return NSSize(width: contentSize.width + _borderWidth*2, height: contentSize.height + _borderWidth*2 + _beakHeight)
        case .Left:             return NSSize(width: contentSize.width + _borderWidth*2 + _beakHeight, height: contentSize.height + _borderWidth*2)
        case .Right:            return NSSize(width: contentSize.width + _borderWidth*2 + _beakHeight, height: contentSize.height + _borderWidth*2)
        case .OnTop:            return NSSize(width: contentSize.width + _borderWidth*2, height: contentSize.height + _borderWidth*2)
        case .WindowCentered:   return NSSize(width: contentSize.width + _borderWidth*2, height: contentSize.height + _borderWidth*2)
        case .WindowTop:        return NSSize(width: contentSize.width + _borderWidth*2, height: contentSize.height + _borderWidth*2)
        }
    }

    /// This view is not opaque
    override var isOpaque: Bool {
        return false
    }

    ///
    /// Draws a line with a 'beak' in it at a certain distance along. Assumes the path's last point
    /// ended at 'from'.
    ///
    func drawLineWithBeak(path: CGMutablePath, from: CGPoint, to: CGPoint, beakPosition: CGFloat = 0.5) {
        // Work out the direction and normal for the line
        let offset  = CGPoint(x: to.x-from.x, y: to.y-from.y)
        let length  = (offset.x*offset.x + offset.y*offset.y).squareRoot()
        let unit    = CGPoint(x: offset.x/length, y: offset.y/length)
        let normal  = CGPoint(x: unit.y, y: unit.x)

        // First part starts from 'from' and moves to the beak position (minus half the beak width)
        let initialLength = (length * beakPosition) - _beakWidth/2.0
        path.addLine(to: CGPoint(x: from.x + unit.x*initialLength, y: from.y + unit.y*initialLength))

        // Beak sticks out from the line
        let beakCenterPoint = length * beakPosition
        let beakNormal      = CGPoint(x: normal.x*_beakHeight, y: normal.y*_beakHeight)
        let beakCenter      = CGPoint(x: from.x + unit.x*beakCenterPoint + beakNormal.x, y: from.y + unit.y*beakCenterPoint + beakNormal.y)
        path.addLine(to: beakCenter)

        // Return back to a point on the line
        let finalLength = (length * beakPosition) + _beakWidth/2.0
        path.addLine(to: CGPoint(x: from.x + unit.x*finalLength, y: from.y + unit.y*finalLength))

        // Draw to the end
        path.addLine(to: to)
    }

    ///
    /// Draws a rounded rectangle covering the given bounds with the specified line with a beak
    ///
    func drawRoundedRectangleWithBeak(path: CGMutablePath, bounds: CGRect, beakSide: UInt32) {
        // Initial rounded rectangle
        path.move(to: CGPoint(x: bounds.minX, y: bounds.maxY-_borderWidth))
        path.addArc(center: CGPoint(x: bounds.minX+_borderWidth, y: bounds.maxY-_borderWidth), radius: _borderWidth, startAngle: CGFloat.pi, endAngle: CGFloat.pi/2, clockwise: true)
        if beakSide == 0 {
            drawLineWithBeak(path: path, from: CGPoint(x: bounds.minX+_borderWidth, y: bounds.maxY), to: CGPoint(x: bounds.maxX-_borderWidth, y: bounds.maxY))
        } else {
            path.addLine(to: CGPoint(x: bounds.maxX-_borderWidth, y: bounds.maxY))
        }

        path.addArc(center: CGPoint(x: bounds.maxX-_borderWidth, y: bounds.maxY-_borderWidth), radius: _borderWidth, startAngle: CGFloat.pi/2, endAngle: 0, clockwise: true)
        if beakSide == 1 {
            drawLineWithBeak(path: path, from: CGPoint(x: bounds.maxX, y: bounds.maxY-_borderWidth), to: CGPoint(x: bounds.maxX, y: bounds.minY+_borderWidth))
        } else {
            path.addLine(to: CGPoint(x: bounds.maxX, y: bounds.minY+_borderWidth))
        }

        path.addArc(center: CGPoint(x: bounds.maxX-_borderWidth, y: bounds.minY+_borderWidth), radius: _borderWidth, startAngle: 0, endAngle: -CGFloat.pi/2, clockwise: true)
        if beakSide == 2 {
            drawLineWithBeak(path: path, from: CGPoint(x: bounds.maxX-_borderWidth, y: bounds.minY), to: CGPoint(x: bounds.minX+_borderWidth, y: bounds.minY))
        } else {
            path.addLine(to: CGPoint(x: bounds.minX+_borderWidth, y: bounds.minY))
        }

        path.addArc(center: CGPoint(x: bounds.minX+_borderWidth, y: bounds.minY+_borderWidth), radius: _borderWidth, startAngle: -CGFloat.pi/2, endAngle: -CGFloat.pi, clockwise: true)
        if beakSide == 3 {
            drawLineWithBeak(path: path, from: CGPoint(x: bounds.minX, y: bounds.minY+_borderWidth), to: CGPoint(x: bounds.minX, y: bounds.maxY-_borderWidth))
        } else {
            path.addLine(to: CGPoint(x: bounds.minX, y: bounds.maxY-_borderWidth))
        }

        path.closeSubpath()
    }

    override func draw(_ dirtyRect: NSRect) {
        let ctxt        = NSGraphicsContext.current!.cgContext
        let bounds      = self.bounds
        var beakBounds  = bounds.insetBy(dx: 2.0, dy: 2.0)

        let path        = CGMutablePath()

        switch direction {
        case .Below:
            beakBounds.size.height -= _beakHeight
            drawRoundedRectangleWithBeak(path: path, bounds: beakBounds, beakSide: 0)
        case .Right:
            beakBounds.size.width -= _beakHeight
            drawRoundedRectangleWithBeak(path: path, bounds: beakBounds, beakSide: 1)
        case .Above:
            beakBounds.size.height -= _beakHeight
            beakBounds.origin.y += _beakHeight
            drawRoundedRectangleWithBeak(path: path, bounds: beakBounds, beakSide: 2)
        case .Left:
            beakBounds.size.width -= _beakHeight
            beakBounds.origin.x += _beakHeight
            drawRoundedRectangleWithBeak(path: path, bounds: beakBounds, beakSide: 3)

        default:
            path.addRoundedRect(in: bounds.insetBy(dx: 2.0, dy: 2.0), cornerWidth: _borderWidth, cornerHeight: _borderWidth)
        }

        ctxt.setFillColor(CGColor(red: 0.25, green: 0.2, blue: 0.2, alpha: 0.9))
        ctxt.addPath(path)
        ctxt.fillPath()

        ctxt.setStrokeColor(CGColor(red: 0.1, green: 0.1, blue: 0.1, alpha: 0.8))
        ctxt.setLineWidth(3.0)
        ctxt.addPath(path)
        ctxt.strokePath()

        ctxt.setStrokeColor(CGColor(red: 0.9, green: 0.9, blue: 0.9, alpha: 0.8))
        ctxt.setLineWidth(2.25)
        ctxt.addPath(path)
        ctxt.strokePath()
    }
}
