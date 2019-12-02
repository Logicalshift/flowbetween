//
//  FloBadgeLayer.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 07/03/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

///
/// A layer used to draw a badge on an item
///
class FloBadgeLayer : CALayer {
    override init() {
        super.init();

        zPosition       = 10000.0;
        frame           = CGRect(x: 0, y: 0, width: 12, height: 12);
        isOpaque        = false;
        contentsScale   = 2.0;
    }

    override init(layer: Any) {
        super.init();

        zPosition       = 10000.0;
        frame           = CGRect(x: 0, y: 0, width: 12, height: 12);
        isOpaque        = false;
        contentsScale   = 2.0;
    }

    required init?(coder aDecoder: NSCoder) {
        super.init();

        zPosition       = 10000.0;
        frame           = CGRect(x: 0, y: 0, width: 12, height: 12);
        isOpaque        = false;
        contentsScale   = 2.0;
    }

    override func draw(in ctx: CGContext) {
        ctx.saveGState();

        ctx.clear(bounds);
        ctx.setFillColor(CGColor(red: 0.3, green: 0.5, blue: 1.0, alpha: 0.8));
        ctx.beginPath();
        ctx.addEllipse(in: bounds);
        ctx.fillPath();

        ctx.restoreGState();
    }
}
