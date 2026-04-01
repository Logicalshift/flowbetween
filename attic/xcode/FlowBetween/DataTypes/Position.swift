//
//  Position.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 03/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Foundation

///
/// Represents a position in a layot
///
enum Position {
    case At(Float64)
    case Offset(Float64)
    case Floating(Float64, FloProperty)
    case Stretch(Float64)
    case Start
    case End
    case After
}
