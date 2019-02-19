//
//  FixedAxis.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 19/02/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Foundation

///
/// Axis that's fixed during scrolling
///
enum FixedAxis : UInt32 {
    case None       = 65536;
    case Horizontal = 0;
    case Vertical   = 1;
    case Both       = 2;
}
