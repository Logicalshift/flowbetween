//
//  FloContainer.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 06/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

///
/// Represents the visibility of the scrollbars
///
enum ScrollBarVisibility {
    case Never
    case Always
    case OnlyIfNeeded
}

///
/// Protocol implemented by views that can act as container views
///
protocol FloContainerView {
    /// The FloView that owns this container view (should be a weak reference)
    var floView: FloView? { get set }

    /// Adds a subview to this container view
    func addContainerSubview(_ subview: NSView)

    /// Sets the layer displayed for the canvas
    func setCanvasLayer(_ layer: CALayer)

    /// Stores the general state of this view
    var viewState : ViewState { get }

    /// The size of the layout area for this view
    var layoutSize : NSSize { get }

    /// Returns this view as an NSView
    var asView : NSView { get }

    /// Event handler: user clicked in the view
    var onClick: (() -> Bool)? { get set }

    /// Event handler: user scrolled/resized so that a particular region is visible
    var onScroll: ((NSRect) -> ())? { get set }

    /// Event handler: value has changed
    var onEditValue: ((PropertyValue) -> ())? { get set }

    /// Event handler: value has been set
    var onSetValue: ((PropertyValue) -> ())? { get set }

    /// Event handler: control has obtained keyboard focus
    var onFocused: (() -> ())? { get set }

    /// Event handler: user has dragged this control
    var onDrag: ((DragAction, CGPoint, CGPoint) -> ())? { get set }

    /// Events handlers when a particular device is used for painting
    var onPaint: [FloPaintDevice: (FloPaintStage, AppPainting) -> ()] { get set }

    /// The affine transform for the canvas layer
    var canvasAffineTransform: CGAffineTransform? { get set }

    /// Event handler: user performed layout on this view
    var performLayout: ((NSSize) -> ())? { get set }

    /// Event handler: The bounds of the container have changed
    var boundsChanged: ((ContainerBounds) -> ())? { get set }

    /// The minimum size of the scroll area for this view
    var scrollMinimumSize: (Float64, Float64) { get set }

    /// The visibility of the horizontal and vertical scroll bars
    var scrollBarVisibility: (ScrollBarVisibility, ScrollBarVisibility) { get set }

    /// Triggers the click event for this view
    func triggerClick()

    /// Triggers the bounds changed event for this view
    func triggerBoundsChanged()

    /// Sets the text label for this view
    func setTextLabel(label: String)

    /// Sets the font size for this view
    func setFontSize(points: Float64)

    /// Sets the foreground colour of the control
    func setForegroundColor(color: NSColor)

    /// Sets the font weight for this view
    func setFontWeight(weight: Float64)

    /// Sets the text alignment for this view
    func setTextAlignment(alignment: NSTextAlignment)

    /// Sets part of the state of this control
    func setState(selector: ViewStateSelector, toProperty: FloProperty)
}

///
/// Returns the names of the classes applied to a particular view
///
func classNamesForView(_ source: FloContainerView) -> [String] {
    var floView = source.floView
    var results = [String]()

    while let view = floView {
        // Add the classes from this view
        results.append(contentsOf: view.viewState.classes)

        // Move up the hierarchy
        floView = floView?.superview
    }

    return results
}

///
/// Returns the Z-Index of a particular view
///
fileprivate func zIndexForView(_ view: NSView) -> CGFloat {
    if let view = view as? FloContainerView {
        return view.viewState.zIndex ?? 0.0
    } else {
        return -1.0
    }
}

///
/// Orders views by their z-index
///
/// This ensures that mouse actions reach higher views first
///
func sortSubviewsByZIndex(_ parentView: NSView) {
    // Fetch the subviews
    var subviews = parentView.subviews

    // Sort them by z-index (so that clicks will reach higher views first)
    subviews.sort(by: { a, b in
        let aZIndex = zIndexForView(a)
        let bZIndex = zIndexForView(b)

        return aZIndex < bZIndex
    })

    // Store the sorted views
    parentView.subviews = subviews
}

///
/// Bubbles an event up from a particular view
///
func bubbleUpEvent(source: NSView, event_handler: (FloContainerView) -> Bool) {
    // Bubble up to the superview
    var bubble_to: NSView? = source

    while true {
        if let bubble_to_view = bubble_to {
            // Try this view
            if let bubble_to = bubble_to_view as? FloContainerView {
                if event_handler(bubble_to) {
                    // Event was handled
                    return
                }
            }

            // Try the superview
            bubble_to = bubble_to_view.superview
        } else {
            // Did not find a target
            return
        }
    }
}
