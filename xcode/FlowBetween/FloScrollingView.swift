//
//  FloScrollingView.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 06/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

public class FloScrollingView : NSScrollView, FloContainerView {
    public required init?(coder: NSCoder) {
        _scrollMinimumSize = (0,0);
        _scrollBarVisibility = (ScrollBarVisibility.OnlyIfNeeded, ScrollBarVisibility.OnlyIfNeeded);
        
        super.init(coder: coder)

        self.documentView = FloEmptyView.init(frame: NSRect(x: 0, y: 0, width: 4000, height: 4000));
        self.documentView?.wantsLayer = false;

        self.wantsLayer             = true;
        self.hasHorizontalScroller  = true;
        self.hasVerticalScroller    = true;
        self.autohidesScrollers     = true;
        
        self.contentView.postsBoundsChangedNotifications = true;
        NotificationCenter.default.addObserver(self, selector: #selector(triggerOnScroll), name: NSView.boundsDidChangeNotification, object: self.contentView);
    }
    
    required public override init(frame frameRect: NSRect) {
        _scrollMinimumSize = (0,0);
        _scrollBarVisibility = (ScrollBarVisibility.OnlyIfNeeded, ScrollBarVisibility.OnlyIfNeeded);

        super.init(frame: frameRect);

        self.documentView = FloEmptyView.init(frame: NSRect(x: 0, y: 0, width: 4000, height: 4000));
        self.documentView?.wantsLayer = false;

        self.wantsLayer             = true;
        self.hasHorizontalScroller  = true;
        self.hasVerticalScroller    = true;
        self.autohidesScrollers     = true;
        
        self.contentView.postsBoundsChangedNotifications = true;
        NotificationCenter.default.addObserver(self, selector: #selector(triggerOnScroll), name: NSView.boundsDidChangeNotification, object: self.contentView);
    }
    
    override public var isOpaque: Bool { get { return false } }

    ///
    /// Adds a subview to this container
    ///
    func addContainerSubview(_ subview: NSView) {
        self.documentView!.addSubview(subview);
    }
    
    ///
    /// Sets the sizing for the document view
    ///
    func layoutDocumentView() {
        // Decide on the size of the document view
        let (minX, minY)    = scrollMinimumSize;
        let contentSize     = contentView.bounds.size;
        
        let sizeX           = CGFloat.maximum(CGFloat(minX), contentSize.width);
        let sizeY           = CGFloat.maximum(CGFloat(minY), contentSize.height);
        
        let newSize         = CGSize(width: sizeX, height: sizeY);
        
        documentView?.setFrameSize(newSize);
        
        // Perform general layout
        self.performLayout?(newSize);

        // Any subviews that are not themselves FloContainers are sized to fill this view
        for subview in self.documentView!.subviews {
            if (subview as? FloContainerView) == nil {
                subview.frame = NSRect(origin: CGPoint(x: 0, y: 0), size: newSize);
            }
        }
    }

    /// The size of the layout area for this view
    var layoutSize : NSSize {
        get {
            if let documentView = documentView {
                return documentView.bounds.size;
            } else {
                let (width, height) = scrollMinimumSize;
                return NSSize(width: width, height: height);
            }
        }
    };

    ///
    /// Containers cause the layout algorithm to run when they are resized
    ///
    override public func setFrameSize(_ newSize: NSSize) {
        super.setFrameSize(newSize);
        
        layoutDocumentView();
        triggerOnScroll();
    }

    fileprivate var _scrollMinimumSize: (Float64, Float64);

    /// The minimum size of the scroll area for this view
    var scrollMinimumSize: (Float64, Float64) {
        get { return _scrollMinimumSize; }
        set(value) {
            _scrollMinimumSize = value;
        }
    }

    fileprivate var _scrollBarVisibility: (ScrollBarVisibility, ScrollBarVisibility);

    /// The visibility of the horizontal and vertical scroll bars
    var scrollBarVisibility: (ScrollBarVisibility, ScrollBarVisibility) {
        get { return _scrollBarVisibility; }
        set(value) {
            _scrollBarVisibility = value;
            
            // Set the scrollbar visibility
            let (horiz, vert) = value;
            switch (horiz) {
            case ScrollBarVisibility.Always, ScrollBarVisibility.OnlyIfNeeded:  self.hasHorizontalScroller = true; break;
            case ScrollBarVisibility.Never:                                     self.hasHorizontalScroller = false; break;
            }

            switch (vert) {
            case ScrollBarVisibility.Always, ScrollBarVisibility.OnlyIfNeeded:  self.hasVerticalScroller = true; break;
            case ScrollBarVisibility.Never:                                     self.hasVerticalScroller = false; break;
            }

            // Cocoa can't auto-hide individually, so we always auto-hide both scrollbars
            switch (value) {
            case (ScrollBarVisibility.OnlyIfNeeded, _), (_, ScrollBarVisibility.OnlyIfNeeded):
                self.autohidesScrollers = true;
                break;
            
            default:
                self.autohidesScrollers = false;
                break;
            }
        }
    }

    /// Returns this view as an NSView
    var asView : NSView { get { return self; } };
    
    /// Event handler: user clicked in the view
    var onClick: (() -> Bool)?;

    /// Event handler: user scrolled/resized so that a particular region is visible
    var _onScroll: ((NSRect) -> ())?;
    var onScroll: ((NSRect) -> ())? {
        get { return _onScroll; }
        set(value) {
            _onScroll = value;
            
            triggerOnScroll();
        }
    }

    /// Event handler: user performed layout on this view
    var performLayout: ((NSSize) -> ())?;
    
    /// Event handler: The bounds of the container have changed
    var boundsChanged: ((ContainerBounds) -> ())?;

    /// Triggers the click event for this view
    func triggerClick() {
        bubble_up_event(source: self, event_handler: { (container) in
            if let onClick = container.onClick {
                return onClick();
            } else {
                return false;
            }
        });
    }
    
    /// Triggers the scroll event for this view
    @objc func triggerOnScroll() {
        // This also changes the bounds of the view
        triggerBoundsChanged();
        
        // Find the area that's visible on screen
        let visibleRect = self.convert(bounds, to: documentView);
        
        // Send the onScroll event
        _onScroll?(visibleRect);
    }

    /// Sets the layer displayed for the canvas
    func setCanvasLayer(_ layer: CALayer) {
        self.documentView!.layer!.addSublayer(layer);
    }


    ///
    /// Triggers the bounds changed event for this view
    ///
    func triggerBoundsChanged() {
        // For scrolling views, we actually trigger for all the subviews of the document view
        var toProcess = [self.documentView!];
        
        while let view = toProcess.popLast() {
            // If the view is a container view, trigger its bounds changed event
            if let view = view as? FloContainerView {
                view.triggerBoundsChanged();
            }
            
            // If the view is not a scrolling view, add its subviews (nested scrolling views will already have triggered the event)
            if !(view is FloScrollingView) {
                for subview in view.subviews {
                    toProcess.append(subview);
                }
            }
        }
    }
}
