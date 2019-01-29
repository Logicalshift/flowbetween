//
//  FloPaintDevice.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 29/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Foundation

///
/// Devices that can have paint requests sent for them
///
/// Corresponds to AppPaintDevice from the Rust side of things
///
enum FloPaintDevice : UInt32 {
    case MouseLeft      = 0;
    case MouseMiddle    = 1;
    case MouseRight     = 2;
    case Pen            = 3;
    case Eraser         = 4;
    case Touch          = 5;
}
