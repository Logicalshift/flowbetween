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
    case At(Float32);
    case Offset(Float32);
    case Stretch(Float32);
    case Start;
    case End;
    case After;
}
