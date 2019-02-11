//
//  DragAction.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 11/02/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Foundation

///
/// Actions that occur during a dragging operation
///
enum DragAction {
    case Start;
    case Continue;
    case Finish;
    case Cancel;
}
